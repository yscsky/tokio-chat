use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::mpsc::{self, Receiver, Sender},
};
use tokio_chat::FromClient;

use crate::FromServer;

pub struct Connection;

impl Connection {
    pub fn run<F>(socket: TcpStream, read_handler: F)
    where
        F: Fn(FromClient, Sender<FromServer>) + Send + 'static,
    {
        let (reader, writer) = socket.into_split();
        let (tx, rx) = mpsc::channel::<FromServer>(100);

        tokio::spawn(async {
            let mut read_task = tokio::spawn(async {
                Self::read_from_client(reader, tx, read_handler).await;
            });
            let mut write_task = tokio::spawn(async {
                Self::write_to_client(writer, rx).await;
            });
            if tokio::try_join!(&mut read_task, &mut write_task).is_err() {
                eprintln!("read_task/write_task terminated");
                read_task.abort();
                write_task.abort();
            }
        });
    }

    async fn read_from_client<F>(reader: OwnedReadHalf, tx: Sender<FromServer>, read_handler: F)
    where
        F: Fn(FromClient, Sender<FromServer>) + Send + 'static,
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
                    let msg: FromClient = serde_json::from_str(&line).unwrap();
                    read_handler(msg, tx.clone());
                }
            }
        }
    }

    async fn write_to_client(writer: OwnedWriteHalf, mut rx: Receiver<FromServer>) {
        let mut buf_writer = BufWriter::new(writer);
        while let Some(msg) = rx.recv().await {
            let mut json = serde_json::to_string(&msg).unwrap();
            json.push('\n');
            if let Err(e) = buf_writer.write_all(json.as_bytes()).await {
                eprintln!("write to client failed: {}", e);
                break;
            }
        }
    }
}
