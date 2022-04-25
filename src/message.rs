use tokio::io::BufReader;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub enum Message {
    Init,
    Terminate,
    Invalid,
}
pub enum Buffer {
    Message(Message),
    End,
    Invalid,
}
pub struct Messenger {
    stream: BufReader<TcpStream>,
}

impl Messenger {
    pub async fn read_buffer(&mut self) -> Result<Vec<u8>, ()> {
        let mut count_buff = [0; 4];
        self.stream.read(&mut count_buff).await.map_err(|_| ())?;
        let count = u32::from_be_bytes(count_buff);
        let mut buffer = vec![0; count as usize];
        self.stream.read(&mut buffer).await.map_err(|_| ())?;
        println!(" -> {:?}", buffer);
        Ok(buffer)
    }
    pub async fn recv(&mut self) -> Result<Buffer, ()> {
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
    pub async fn send(&mut self, mes: Message) -> Result<(), ()> {
        let buffer = match mes {
            Message::Init => "init",
            Message::Terminate => "terminate",
            Message::Invalid => "message no in correct format",
        }
        .as_bytes();

        self.stream.write_all(buffer).await.map_err(|_| ())?;
        Ok(())
    }
    pub async fn close(&mut self) -> Result<(), ()> {
        self.stream.shutdown().await.map_err(|_| ())?;
        Ok(())
    }
}

impl From<BufReader<TcpStream>> for Messenger {
    fn from(stream: BufReader<TcpStream>) -> Messenger {
        Messenger { stream }
    }
}
