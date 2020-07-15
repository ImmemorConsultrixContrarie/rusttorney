use crate::config::Config;
use bytes::{BytesMut, Bytes};
use std::{fmt::Debug, str};
use tokio::{
    io::{AsyncReadExt},
    net::{TcpListener},
};
use std::{
    io::{BufRead, Cursor, Seek, SeekFrom},
};
use tokio::{
    stream::StreamExt,
};
use tokio_util::codec::{Decoder, FramedRead};
use std::{marker::PhantomData, borrow::{BorrowMut, Cow}};
use crate::networking::Command;

#[derive(Debug)]
pub enum ClientCommand<'a> {
    Handshake(&'a str),                             // HI#<hdid:String>#%
    ClientVersion(u32, &'a str, &'a str),           // ID#<pv:u32>#<software:String>#<version:String>#%
    KeepAlive,                                      // CH
    AskListLengths,                                 // askchaa
    AskListCharacters,                              // askchar
    CharacterList(u32),                             // AN#<page:u32>#%
    EvidenceList(u32),                              // AE#<page:u32>#%
    MusicList(u32),                                 // AM#<page:u32>#%
    AO2CharacterList,                               // AC#%
    AO2MusicList,                                   // AM#%
    AO2Ready,                                       // RD#%
    SelectCharacter(u32, u32, &'a str),             // CC<client_id:u32>#<char_id:u32#<hdid:String>#%
    ICMessage,                                      // MS
    OOCMessage(&'a str, &'a str),                   // CT#<name:String>#<message:String>#%
    PlaySong(u32, u32),                             // MC#<song_name:u32>#<???:u32>#%
    WTCEButtons(&'a str),                           // RT#<type:String>#%
    SetCasePreferences(&'a str, CasePreferences),   // SETCASE#<cases:String>#<will_cm:boolean>#<will_def:boolean>#<will_pro:boolean>#<will_judge:boolean>#<will_jury:boolean>#<will_steno:boolean>#%
    CaseAnnounce(&'a str, CasePreferences),         // CASEA
    Penalties(u32, u32),                            // HP#<type:u32>#<new_value:u32>#%
    AddEvidence(EvidenceArgs<'a>),                  // PE#<name:String>#<description:String>#<image:String>#%
    DeleteEvidence(u32),                            // DE#<id:u32>#%
    EditEvidence(u32, EvidenceArgs<'a>),            // EE#<id:u32>#<name:String>#<description:String>#<image:String>#%
    CallModButton(Option<&'a str>),                 // ZZ?#<reason:String>?#%
}

impl<'a> Command<'a> for ClientCommand<'a> {
    fn from_protocol(name: &'a str, args: impl Iterator<Item = &'a str>) -> Result<Self, anyhow::Error> {
        Ok(Self::ICMessage)
    }
    fn handle(&self) -> futures::future::BoxFuture<'static, ()> {
        todo!()
    }
}

#[derive(Debug)]
pub struct EvidenceArgs<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub image: &'a str,
}

#[derive(Debug)]
pub struct CasePreferences {
    pub cm: bool,
    pub def: bool,
    pub pro: bool,
    pub judge: bool,
    pub jury: bool,
    pub steno: bool
}

#[derive(Debug)]
pub struct AOMessage<'a> {
    pub command: ClientCommand<'a>,
}

pub struct AOMessageCodec<'a> {
    _phantom: PhantomData<&'a ()>,
}

impl<'a> AOMessageCodec<'a> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<'a> Decoder for AOMessageCodec<'a> {
    type Item = ClientCommand<'a>;
    type Error = anyhow::Error;

    fn decode(
        &mut self,
        mut src: &mut BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        let magic_b = src.iter().position(|&byte| byte == b'#');
        if let Some(i) = magic_b {
            let cmd = src.split_to(i);

            let cmd_name = match String::from_utf8_lossy(&cmd) {
                Cow::Owned(mut own) => {
                    own.retain(|c| c == '�');
                    Cow::Owned(own)
                },
                b @ Cow::Borrowed(_) => b
            };
            src.split_to(1);
            
            let protocol_end = src.iter().rposition(|&b| b == b'%');

            if let Some(i) = protocol_end {
                let args = src.split_to(i - 2);

                return Ok(Some(Command::from_protocol(&cmd_name, args.as_ref().split(|&b| b == b'#').map(|s| String::from_utf8_lossy(s).as_ref()))?));
            }
        }

        src.clear();

        Ok(None)
    }
}

pub struct AOServer<'a> {
    config: Config<'a>
}

impl<'a> AOServer<'a> {
    pub fn new(config: Config<'a>) -> anyhow::Result<Self> {
        Ok(Self {
            config
        })
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        log::info!("Starting up the server...");
        let addr = format!("127.0.0.1:{}", self.config.general.port);
        log::debug!("Binding to address: {}", &addr);

        let mut listener = TcpListener::bind(addr).await?;

        loop {
            let (mut socket, c) = listener.accept().await?;
            log::debug!("got incoming connection from: {:?}", &c);

            tokio::spawn(async move {
                let mut buf = [0; 64];

                loop {
                    let n = socket
                        .read(&mut buf)
                        .await
                        .expect("Failed to read data!");

                    if n == 0 {
                        break;
                    }
                }

                let mut fr =
                    FramedRead::new(buf.as_ref(), AOMessageCodec::new());
                while let Some(Ok(message)) = fr.next().await {
                    log::debug!("Got message: {:?}", message)
                }
            });
        }
    }
}