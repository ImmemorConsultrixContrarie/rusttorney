use crate::config::Config;
use bytes::{BufMut, BytesMut};
use std::{borrow::Cow, fmt::Debug, str};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, ToSocketAddrs},
};
use std::fmt::Formatter;
use std::{
    io::{BufRead, Cursor, Seek, SeekFrom},
    str::FromStr,
};
use tokio::{
    io::{AsyncBufReadExt, Error, ErrorKind},
    stream::StreamExt,
};
use tokio_util::codec::{Decoder, Encoder, FramedRead};
use std::path::PathBuf;
use crate::networking::Command;

#[derive(Debug)]
pub enum ClientCommand {
    Handshake(String),          // HI#<hdid:String>#%
    ClientVersion(u32, String, String),      // ID#<pv:u32>#<software:String>#<version:String>#%
    KeepAlive,          // CH
    AskListLengths,     // askchaa
    AskListCharacters,  // askchar
    CharacterList(u32),      // AN#<page:u32>#%
    EvidenceList(u32),       // AE#<page:u32>#%
    MusicList(u32),          // AM#<page:u32>#%
    AO2CharacterList,   // AC#%
    AO2MusicList,       // AM#%
    AO2Ready,           // RD#%
    SelectCharacter(u32, u32, String),    // CC<client_id:u32>#<char_id:u32#<hdid:String>#%
    ICMessage,          // MS
    OOCMessage(String, String),         // CT#<name:String>#<message:String>#%
    PlaySong(u32, u32),           // MC#<song_name:u32>#<???:u32>#%
    WTCEButtons(String),        // RT#<type:String>#%
    SetCasePreferences(String, CasePreferences), // SETCASE#<cases:String>#<will_cm:boolean>#<will_def:boolean>#<will_pro:boolean>#<will_judge:boolean>#<will_jury:boolean>#<will_steno:boolean>#%
    CaseAnnounce(String, CasePreferences),       // CASEA
    Penalties(u32, u32),          // HP#<type:u32>#<new_value:u32>#%
    AddEvidence(EvidenceArgs),        // PE#<name:String>#<description:String>#<image:String>#%
    DeleteEvidence(u32),     // DE#<id:u32>#%
    EditEvidence(u32, EvidenceArgs),       // EE#<id:u32>#<name:String>#<description:String>#<image:String>#%
    CallModButton(Option<String>),      // ZZ?#<reason:String>?#%
}

impl Command for ClientCommand {
    fn from_protocol(name: String, mut args: Vec<String>) -> Result<Self, anyhow::Error> {
        let args_len = args.len();
        match name.as_str() {
            "HI" => {
                if args_len != 1 {
                    anyhow::bail!("Amount of arguments for command HANDSHAKE does not match!")
                }

                Ok(Self::Handshake(args.remove(0)))
            },
            _ => Ok(Self::ICMessage)
        }
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
    pub steno: bool
}

#[derive(Debug)]
pub struct AOMessage {
    pub command: ClientCommand,
}

pub struct AOMessageCodec;

impl Decoder for AOMessageCodec {
    type Item = AOMessage;
    type Error = anyhow::Error;

    fn decode(
        &mut self,
        mut src: &mut BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }
        let mut reader = Cursor::new(&src);
        let mut cmd_buf = Vec::with_capacity(8);
        let cmd_len = BufRead::read_until(&mut reader, b'#', &mut cmd_buf)?;

        if cmd_len == 0 {
            log::error!("Invalid protocol?!");
            anyhow::bail!("Invalid protocol!")
        }

        cmd_buf.truncate(cmd_len.saturating_sub(1));
        let command_name = String::from_utf8_lossy(&cmd_buf).replace("�", "");

        reader.seek(SeekFrom::Start(cmd_len as u64))?;
        let mut splitted = BufRead::split(reader, b'#')
            .map(|l| String::from_utf8_lossy(&l.unwrap()).replace("�", ""));
        let mut args: Vec<String> = splitted.collect();
        let mut end_header_index = 0;

        for (i, arg) in args.iter().enumerate() {
            if arg.chars().next().unwrap() == '%' {
                end_header_index = i;
                break;
            } else {
                continue;
            }
        }

        args.remove(end_header_index);

        src.clear();

        Ok(Some(AOMessage { command: Command::from_protocol(command_name, args)? }))
    }
}

pub struct AOServer {
    config: Config
}

impl AOServer {
    pub fn new() -> anyhow::Result<Self> {
        let config_path = PathBuf::from("./config/config.toml");
        log::debug!("Getting config from default path: {:?}", &config_path);
        let config: Config = toml::from_str(&std::fs::read_to_string(&config_path)?)?;

        Ok(Self {
            config
        })
    }
}

impl AOServer {
    pub async fn run(&self) -> anyhow::Result<()> {
        log::info!("Starting up the server...");
        let addr = format!("127.0.0.1:{}", self.config.general.port);
        log::debug!("Binding to address: {}", &addr);

        let mut listener = TcpListener::bind(addr).await?;

        loop {
            let (mut socket, c) = listener.accept().await?;
            log::debug!("got incoming connection from: {:?}", &c);

            tokio::spawn(async move {
                let mut buf = [0; 1024];

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
                    FramedRead::new(buf.as_ref(), AOMessageCodec);
                while let Some(Ok(message)) = fr.next().await {
                    log::debug!("Got message: {:?}", message)
                }
            });
        }
    }

    async fn dispatch(data: String) -> anyhow::Result<()> {
        log::debug!("Unparsed data: {:?}", &data);
        Ok(())
    }
}
