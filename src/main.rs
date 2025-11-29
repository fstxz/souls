use std::{
    collections::HashMap,
    net::{SocketAddr, SocketAddrV4},
    sync::{Arc, RwLock},
};

use argh::FromArgs;
use smol::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::{
    buffer::{BufferReader, BufferWriter},
    db::Db,
    messages::{ServerMessage, UserStatus},
};

mod buffer;
mod db;
mod messages;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

#[derive(Default)]
struct ConnectedUsers {
    users: HashMap<SocketAddrV4, User>,
}

pub struct Context<'a, 'b> {
    socket_addr: SocketAddrV4,
    reader: &'a mut BufferReader<'b>,
    db: &'a Db,
    users: Arc<RwLock<ConnectedUsers>>,
}

struct User {
    addr: SocketAddrV4,
    name: String,
    status: UserStatus,
}

/// Soulseek server.
#[derive(FromArgs)]
struct Args {
    /// port to listen on (default: 2242)
    #[argh(option, short = 'p', default = "2242")]
    port: u16,
}

fn main() -> Result<()> {
    simple_logger::init()?;
    let args = argh::from_env::<Args>();

    setup_db()?;

    smol::block_on(async {
        let users = Arc::new(RwLock::new(ConnectedUsers::default()));
        let listener = TcpListener::bind(("127.0.0.1", args.port)).await?;
        log::info!("Listening on port {}", args.port);

        loop {
            let (stream, peer_addr) = listener.accept().await?;
            log::info!("Accepted client: {peer_addr}");

            smol::spawn(handle_client(stream, users.clone())).detach();
        }
    })
}

async fn handle_client(mut socket: TcpStream, users: Arc<RwLock<ConnectedUsers>>) -> Result<()> {
    let SocketAddr::V4(socket_addr) = socket.peer_addr()? else {
        return Err("IP is not v4".into());
    };

    let db = Db::open()?;

    let mut header = [0u8; 4];
    loop {
        let result = socket.read_exact(&mut header).await;

        match result {
            Ok(_) => {
                let message_length = u32::from_le_bytes(header) as usize;

                let mut body = vec![0; message_length];
                socket.read_exact(&mut body).await?;

                match parse_message(socket_addr, &body, &db, users.clone()) {
                    Ok(Some(response)) => {
                        let mut writer = BufferWriter::new();
                        writer.write_byte_array(&response);

                        socket.write_all(writer.buffer()).await?;
                    }
                    // Not sending a response
                    Ok(None) => {
                        continue;
                    }
                    Err(e) => {
                        log::error!("Error: {e}");
                        continue;
                    }
                }
            }
            _ => break,
        }
    }

    users.write().unwrap().users.remove(&socket_addr);

    log::info!("Client disconnected.");
    Ok(())
}

fn parse_message(
    socket_addr: SocketAddrV4,
    buffer: &[u8],
    db: &Db,
    users: Arc<RwLock<ConnectedUsers>>,
) -> Result<Option<Vec<u8>>> {
    let mut reader = BufferReader::new(buffer);
    let code = reader.read_u32()?;

    match ServerMessage::from_code(code) {
        Some(server_message) => {
            let mut context = Context {
                socket_addr,
                reader: &mut reader,
                db,
                users,
            };
            let response = server_message.process(&mut context);
            assert!(reader.is_empty());
            response
        }
        _ => {
            log::warn!("Unknown server message code {code}, continuing");
            Ok(None)
        }
    }
}

fn setup_db() -> Result<()> {
    let db = Db::open()?;
    if !db.table_exists()? {
        db.create_table()?;
    }
    Ok(())
}
