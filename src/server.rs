use crate::config::CONFIG;
use bytes::{BufMut, BytesMut};
use std::{borrow::Cow, fmt::Debug, str};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

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

enum DecodeState {
    Command,
    Arguments,
}

use serde::export::Formatter;
use std::io::Cursor;
use tokio::{
    io::{AsyncBufReadExt, Error},
    stream::StreamExt,
};
use tokio_util::codec::{Decoder, Encoder, FramedRead};

#[derive(Debug)]
pub struct AOMessage {
    pub command: Command,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub struct AOMessageError;

impl std::fmt::Display for AOMessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ao message error!")
    }
}

impl std::error::Error for AOMessageError {}

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
    type Error = AOMessageError;

    fn decode(
        &mut self,
        mut src: &mut BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        match self.state {
            DecodeState::Command => {
                let mut cmd_buf = Vec::with_capacity(8);
                let mut reader = Cursor::new(src);
                let cmd = reader.read_until(b'#', &mut cmd_buf).await?;
                assert!(cmd >= 2 && cmd <= 8);
                log::debug!("Cmd buf: {:?}", &cmd_buf);
                Ok(Some(AOMessage {
                    command: Command::Handshake,
                    args: vec![],
                }))
            }
            DecodeState::Arguments => Ok(Some(AOMessage {
                command: Command::Handshake,
                args: vec![],
            })),
        }
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

pub struct AOServer;

impl AOServer {
    pub async fn run() -> anyhow::Result<()> {
        log::info!("Starting up the server...");
        let mut addr = String::with_capacity(16);
        let config = CONFIG.get().unwrap();
        addr.push_str("127.0.0.1:");
        addr.push_str(&config.general.port.to_string());
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

                let mut fr = FramedRead::new(&mut buf, AOMessageCodec::new());
                fr.next()
            });
        }
    }

    async fn dispatch(data: String) -> anyhow::Result<()> {
        log::debug!("Unparsed data: {:?}", &data);
        Ok(())
    }
}
