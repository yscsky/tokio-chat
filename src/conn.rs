use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

use crate::ChatResult;

pub struct Connection {
    buf_write: BufWriter<OwnedWriteHalf>,
    buf_read: BufReader<OwnedReadHalf>,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Connection {
        let (read, write) = socket.into_split();
        Connection {
            buf_write: BufWriter::new(write),
            buf_read: BufReader::new(read),
        }
    }

    pub async fn read(&mut self) -> ChatResult<String> {
        let mut line = String::new();
        self.buf_read.read_line(&mut line).await?;
        Ok(line)
    }

    pub async fn wirte(&mut self, msg: &mut String) -> ChatResult<()> {
        msg.push('\n');
        self.buf_write.write_all(msg.as_bytes()).await?;
        self.buf_write.flush().await?;
        Ok(())
    }

    pub async fn send<T>(&mut self, msg: &T) -> ChatResult<()>
    where
        T: Serialize,
    {
        let mut json = serde_json::to_string(&msg)?;
        self.wirte(&mut json).await
    }

    pub async fn receive<T>(&mut self) -> ChatResult<T>
    where
        T: DeserializeOwned,
    {
        let line = self.read().await?;
        let p = serde_json::from_str::<T>(&line)?;
        Ok(p)
    }
}
