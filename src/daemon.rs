use std::path::PathBuf;

use tokio::{io::BufReader, net::TcpListener};

use crate::message::{Buffer, Message};

use super::Messenger;

struct Server {
    socket_server: TcpListener,
}

impl Server {
    async fn accept(&self) -> Result<Messenger, ()> {
        let (stream, _) = self.socket_server.accept().await.map_err(|_| ())?;
        println!("HELLO\n");
        Ok(Messenger::from(BufReader::new(stream)))
    }

    async fn new(port: u16) -> Result<Server, ()> {
        let socket_server = TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .map_err(|_| ())?;
        Ok(Server { socket_server })
    }
}

async fn handle_client(#[allow(unused)] path: PathBuf, client: Messenger) -> Result<(), ()> {
    let mut client = client;
    while let Ok(buf) = client.recv().await {
        match buf {
            Buffer::Invalid => client.send(Message::Invalid).await?,
            Buffer::End => {
                client.close().await?;
                break;
            }
            Buffer::Message(mes) => match mes {
                Message::Init => {
                    client.send(Message::Terminate).await?;
                    client.close().await?;
                    break;
                }
                _ => client.send(Message::Invalid).await?,
            },
        }
    }
    Ok(())
}

pub async fn run_daemon(path: PathBuf, port: u16) -> Result<(), ()> {
    let server = Server::new(port).await?;
    loop {
        let client = server.accept().await?;
        tokio::spawn(handle_client(path.clone(), client));
    }
}
