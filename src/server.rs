use crate::{config::Config, networking::Command};
use bytes::{Buf, Bytes, BytesMut};
use futures::{FutureExt, SinkExt};
use std::{
    borrow::{BorrowMut, Cow},
    char::REPLACEMENT_CHARACTER,
    fmt::{Debug, Display},
    io::{BufRead, Cursor, Seek, SeekFrom},
    marker::PhantomData,
    str,
    str::FromStr,
};
use tokio::{io::AsyncReadExt, net::TcpListener, stream::StreamExt};
use tokio_util::codec::{Decoder, FramedRead, Encoder};
use futures::io::Error;
use std::fmt::Write;
use std::sync::Arc;
use futures::lock::Mutex;

const MAGIC_SEPARATOR: u8 = b'#';
const MAGIC_END: u8 = b'%';

#[rustfmt::skip]
#[derive(Debug)]
pub enum ClientCommand {
    Handshake(String),                  // HI#<hdid:String>#%
    ClientVersion(u32, String, String), /* ID#<pv:u32>#<software:String>#
                                         * <version:String>#% */
    KeepAlive,                                   // CH
    AskListLengths,                              // askchaa
    AskListCharacters,                           // askchar
    CharacterList(u32),                          // AN#<page:u32>#%
    EvidenceList(u32),                           // AE#<page:u32>#%
    MusicList(u32),                              // AM#<page:u32>#%
    AO2CharacterList,                            // AC#%
    AO2MusicList,                                // AM#%
    AO2Ready,                                    // RD#%
    SelectCharacter(u32, u32, String),           /* CC<client_id:u32>#
                                                  * <char_id:u32#<hdid:
                                                  * String>#% */
    ICMessage,                                   // MS
    OOCMessage(String, String),                  /* CT#<name:String>#
                                                  * <message:String>#% */
    PlaySong(u32, u32),  // MC#<song_name:u32>#<???:u32>#%
    WTCEButtons(String), // RT#<type:String>#%
    SetCasePreferences(String, CasePreferences), /* SETCASE#<cases:String>#<will_cm:boolean>#<will_def:boolean>#<will_pro:boolean>#<will_judge:boolean>#<will_jury:boolean>#<will_steno:boolean>#% */
    CaseAnnounce(String, CasePreferences),       // CASEA
    Penalties(u32, u32),                         /* HP#<type:u32>#
                                                  * <new_value:u32>#% */
    AddEvidence(EvidenceArgs), /* PE#<name:String>#<description:String>#
                                * <image:String>#% */
    DeleteEvidence(u32),             // DE#<id:u32>#%
    EditEvidence(u32, EvidenceArgs), /* EE#<id:u32>#<name:String>#
                                      * <description:String>#<image:
                                      * String>#% */
    CallModButton(Option<String>), // ZZ?#<reason:String>?#%
}

impl Command for ClientCommand {
    fn from_protocol<'a>(
        name: String,
        mut args: impl Iterator<Item = String>,
    ) -> Result<Self, anyhow::Error> {
        // let args = &mut args;
        let on_err = || {
            anyhow::anyhow!(
                "Amount of arguments for command {} does not match!",
                &name
            )
        };

        fn next<'a, E, T, F>(
            mut args: impl Iterator<Item = String>,
            on_err: F,
        ) -> Result<T, anyhow::Error>
        where
            E: Display,
            T: FromStr<Err = E>,
            F: Fn() -> anyhow::Error,
        {
            args.next()
                .ok_or_else(on_err)
                .map(|s| s.parse::<T>().map_err(|e| anyhow::anyhow!("{}", e))).and_then(std::convert::identity)
        }

        match name.as_str() {
            "HI" => {
                let res = Ok(Self::Handshake(next(&mut args, on_err)?));
                if args.next().is_some() {
                    return Err(on_err());
                }
                res
            },
            _ => Err(on_err()),
        }
    }
    fn handle(&self) -> futures::future::BoxFuture<'static, ()> {
        todo!()
    }
}

#[derive(Debug)]
pub struct EvidenceArgs {
    pub name: String,
    pub description: String,
    pub image: String,
}

#[derive(Debug)]
pub struct CasePreferences {
    pub cm: bool,
    pub def: bool,
    pub pro: bool,
    pub judge: bool,
    pub jury: bool,
    pub steno: bool,
}

#[derive(Debug)]
pub struct AOMessage {
    pub command: ClientCommand,
}

pub struct AOMessageCodec;

impl Decoder for AOMessageCodec {
    type Item = ClientCommand;
    type Error = anyhow::Error;

    fn decode(
        &mut self,
        mut src: &mut BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        let magic_b = src.iter().position(|&byte| byte == MAGIC_SEPARATOR);
        if let Some(i) = magic_b {
            let cmd = src.split_to(i);

            let cmd_name = ignore_ill_utf8(&cmd);
            src.advance(1);

            let protocol_end = src.iter().rposition(|&b| b == MAGIC_END);

            if let Some(i) = protocol_end {
                let args = src.split_to(i - 2);

                src.clear();

                return Ok(Some(Command::from_protocol(
                    cmd_name,
                    args.as_ref()
                        .split(|&b| b == MAGIC_SEPARATOR)
                        .map(|s| ignore_ill_utf8(s)),
                )?));
            }
        }

        Ok(None)
    }
}

fn ignore_ill_utf8(v: &[u8]) -> String {
    let str = String::from_utf8_lossy(&v);

    match str {
        Cow::Owned(mut own) => {
            own.retain(|c| c != REPLACEMENT_CHARACTER);
            own
        }
        Cow::Borrowed(brw) => brw.to_owned(),
    }
}

pub struct AOServer<'a> {
    config: Config<'a>,
}

impl<'a> AOServer<'a> {
    pub fn new(config: Config<'a>) -> anyhow::Result<Self> {
        Ok(Self { config })
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        use futures::StreamExt;

        log::info!("Starting up the server...");
        let addr = format!("127.0.0.1:{}", self.config.general.port);
        log::info!("Binding to address: {}", &addr);

        let mut listener = TcpListener::bind(addr).await?;

        loop {
            let (mut socket, c) = listener.accept().await?;
            log::debug!("got incoming connection from: {:?}", &c);

            let msg_stream = AOMessageCodec.framed(socket);

            tokio::spawn(msg_stream.for_each(move |msg| {
                async move {
                    match msg {
                        Ok(msg) => {
                            log::debug!("Got message: {:?}", msg);
                        },
                        Err(err) => {
                            log::error!("Got error: {:?}", err)
                        }
                    }
                }
            }));
        }
    }
}
