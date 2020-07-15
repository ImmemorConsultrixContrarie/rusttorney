use crate::config::Config;
use bytes::{BufMut, BytesMut};
use std::{borrow::Cow, fmt::Debug, str};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

#[derive(Debug)]
pub enum Command {
    Handshake,          // HI
    ClientVersion,      // ID
    KeepAlive,          // CH
    AskListLengths,     // askchaa
    AskListCharacters,  // askchar
    CharacterList,      // AN
    EvidenceList,       // AE
    MusicList,          // AM
    AO2CharacterList,   // RC
    AO2Ready,           // RD
    SelectCharacter,    // CC
    ICMessage,          // MS
    OOCMessage,         // CT
    PlaySong,           // MC
    WTCEButtons,        // RT
    SetCasePreferences, // SETCASE
    CaseAnnounce,       // CASEA
    Penalties,          // HP
    AddEvidence,        // PE
    DeleteEvidence,     // DE
    EditEvidence,       // EE
    CallModButton,      // ZZ
    KickWithGuard,      // opKICK
    BanWithGuard,       // opBAN
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Command::*;
        match s {
            "HI" => Ok(Handshake),
            "ID" => Ok(ClientVersion),
            "CH" => Ok(KeepAlive),
            "askchaa" => Ok(AskListLengths),
            "askchar" => Ok(AskListCharacters),
            "AN" => Ok(CharacterList),
            "AE" => Ok(EvidenceList),
            "AM" => Ok(MusicList),
            "RC" => Ok(AO2CharacterList),
            "RD" => Ok(AO2Ready),
            "CC" => Ok(SelectCharacter),
            "MS" => Ok(ICMessage),
            "CT" => Ok(OOCMessage),
            "MC" => Ok(PlaySong),
            "RT" => Ok(WTCEButtons),
            "SETCASE" => Ok(SetCasePreferences),
            "CASEA" => Ok(CaseAnnounce),
            "HP" => Ok(Penalties),
            "PE" => Ok(AddEvidence),
            "DE" => Ok(DeleteEvidence),
            "EE" => Ok(EditEvidence),
            "ZZ" => Ok(CallModButton),
            "opKICK" => Ok(KickWithGuard),
            "opBAN" => Ok(BanWithGuard),
            _ => anyhow::bail!("Invalid command!"),
        }
    }
}

enum DecodeState {
    Command,
    Arguments,
}

use serde::export::Formatter;
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

#[derive(Debug)]
pub struct AOMessage {
    pub command: Command,
    pub args: Vec<String>,
}

pub struct AOMessageCodec {
    state: DecodeState,
}

impl AOMessageCodec {
    pub fn new() -> Self {
        Self { state: DecodeState::Command }
    }
}

impl Decoder for AOMessageCodec {
    type Item = AOMessage;
    type Error = anyhow::Error;

    fn decode(
        &mut self,
        mut src: &mut BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        let mut reader = Cursor::new(src);
        let mut cmd_buf = Vec::with_capacity(8);
        let cmd_len = BufRead::read_until(&mut reader, b'#', &mut cmd_buf)?;

        if cmd_len == 0 {
            log::debug!("Invalid protocol?!");
            anyhow::bail!("Invalid protocol!")
        }

        cmd_buf.truncate(cmd_len.saturating_sub(1));
        let command: Command = unsafe {
            // We are sure that the commands will *always* be valid utf8
            String::from_utf8_unchecked(cmd_buf)
        }
        .parse()?;

        reader.seek(SeekFrom::Start(cmd_len as u64))?;
        let mut splitted = BufRead::split(reader, b'#')
            .map(|l| String::from_utf8(l.unwrap()).unwrap());
        let mut args: Vec<String> = splitted.collect();
        let mut end_header_index = 0;

        for (i, arg) in args.iter().enumerate() {
            if arg.chars().nth(0).unwrap() == '%' {
                end_header_index = i;
                break;
            } else {
                continue;
            }
        }

        args.remove(end_header_index);

        Ok(Some(AOMessage { command, args }))
    }
}

fn clean_utf8(mut bytes: &[u8]) -> String {
    let mut output = String::new();

    loop {
        match str::from_utf8(bytes) {
            Ok(s) => {
                output.push_str(s);
                return output;
            }
            Err(e) => {
                let (good, bad) = bytes.split_at(e.valid_up_to());

                if !good.is_empty() {
                    let s = unsafe { str::from_utf8_unchecked(good) };
                    output.push_str(s);
                }

                if bad.is_empty() {
                    return output;
                }

                output.push_str("");

                bytes = &bad[1..];
            }
        }
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
                    FramedRead::new(buf.as_ref(), AOMessageCodec::new());
                let message = fr.next().await;
                log::debug!("Message: {:?}", message)
            });
        }
    }

    async fn dispatch(data: String) -> anyhow::Result<()> {
        log::debug!("Unparsed data: {:?}", &data);
        Ok(())
    }
}
