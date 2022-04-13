use std::path::PathBuf;
use tokio::io::BufReader;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

enum Message {
    Init,
    Terminate,
    Invalid,
}
enum Buffer {
    Message(Message),
    End,
    Invalid,
}
struct Client {
    stream: BufReader<TcpStream>,
}

struct Server {
    socket_server: TcpListener,
}

impl Server {
    async fn accept(&self) -> Result<Client, ()> {
        let (stream, _) = self.socket_server.accept().await.map_err(|_| ())?;
        println!("HELLO\n");
        Ok(Client {
            stream: BufReader::new(stream),
        })
    }

    async fn new(port: u16) -> Result<Server, ()> {
        let socket_server = TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .map_err(|_| ())?;
        Ok(Server { socket_server })
    }
}

impl Client {
    async fn read_buffer(&mut self) -> Result<Vec<u8>, ()> {
        let mut count_buff = [0; 4];
        self.stream.read(&mut count_buff).await.map_err(|_| ())?;
        let count = u32::from_be_bytes(count_buff);
        let mut buffer = vec![0; count as usize];
        self.stream.read(&mut buffer).await.map_err(|_| ())?;
        println!(" -> {:?}", buffer);
        Ok(buffer)
    }
    async fn recv(&mut self) -> Result<Buffer, ()> {
        let buffer = self.read_buffer().await?;
        if buffer.len() == 0 {
            Ok(Buffer::End)
        } else if buffer == "init".to_string().as_bytes() {
            Ok(Buffer::Message(Message::Init))
        } else if buffer == "terminate".to_string().as_bytes() {
            Ok(Buffer::Message(Message::Terminate))
        } else {
            Ok(Buffer::Invalid)
        }
    }
    async fn send(&mut self, mes: Message) -> Result<(), ()> {
        let buffer = match mes {
            Message::Init => "init",
            Message::Terminate => "terminate",
            Message::Invalid => "message no in correct format",
        }
        .as_bytes();

        self.stream.write_all(buffer).await.map_err(|_| ())?;
        Ok(())
    }
    async fn close(&mut self) -> Result<(), ()> {
        self.stream.shutdown().await.map_err(|_| ())?;
        Ok(())
    }
}

async fn handle_client(#[allow(unused)] path: PathBuf, client: Client) -> Result<(), ()> {
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
