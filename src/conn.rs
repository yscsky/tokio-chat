use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::mpsc::{self, Receiver, Sender},
};

pub struct Connection;

impl Connection {
    pub async fn run<R, S>(socket: TcpStream, sender: Sender<R>) -> Sender<S>
    where
        R: DeserializeOwned + Send + 'static,
        S: Serialize + Send + 'static,
    {
        let (reader, writer) = socket.into_split();
        let (tx, rx) = mpsc::channel::<S>(100);
        let ret = tx.clone();

        tokio::spawn(async {
            let mut read_task = tokio::spawn(async {
                Self::read(reader, sender).await;
            });
            let mut write_task = tokio::spawn(async {
                Self::write(writer, rx).await;
            });
            if tokio::try_join!(&mut read_task, &mut write_task).is_err() {
                eprintln!("read_task/write_task terminated");
                read_task.abort();
                write_task.abort();
            }
        });

        ret
    }

    async fn read<R>(reader: OwnedReadHalf, sender: Sender<R>)
    where
        R: DeserializeOwned,
    {
        let mut buf_reader = BufReader::new(reader);
        loop {
            let mut line = String::new();
            match buf_reader.read_line(&mut line).await {
                Err(e) => {
                    eprintln!("read from client error: {e}");
                    break;
                }
                Ok(0) => {
                    println!("client closed");
                    break;
                }
                Ok(_) => {
                    // println!("read: {}", line);
                    let msg: R = serde_json::from_str(&line).unwrap();
                    sender.send(msg).await.unwrap();
                }
            }
        }
    }

    async fn write<S>(writer: OwnedWriteHalf, mut rx: Receiver<S>)
    where
        S: Serialize,
    {
        let mut buf_writer = BufWriter::new(writer);
        while let Some(msg) = rx.recv().await {
            let mut json = serde_json::to_string(&msg).unwrap();
            json.push('\n');
            // println!("write: {}", json);
            buf_writer.write_all(json.as_bytes()).await.unwrap();
            buf_writer.flush().await.unwrap();
        }
    }
}
