use tokio::io::BufReader;

use tokio::net::TcpStream;

use crate::message::{Buffer, Message};
use crate::Messenger;

async fn connect(address: &str, port: u16) -> Result<Messenger, ()> {
    let stream = TcpStream::connect(format!("{}:{}", address, port))
        .await
        .map_err(|_| ())?;
    let stream = BufReader::new(stream);
    Ok(Messenger::from(stream))
}

pub async fn handshake(address: &str, port: u16) -> Result<bool, ()> {
    let mut messenger = connect(address, port).await?;
    messenger.send(Message::Init).await?;
    let mes = messenger.recv().await?;
    match mes {
        Buffer::Invalid | Buffer::End => return Ok(false),
        Buffer::Message(mes) => match mes {
            Message::Terminate => Ok(true),
            _ => Ok(false),
        },
    }
}
