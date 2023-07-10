use std::{env, future::Future, sync::Arc};

use group::GroupTable;
use tokio::{net::TcpListener, signal, sync::mpsc};
use tokio_chat::{conn::Connection, ChatResult, FromClient, FromServer};

mod group;

#[tokio::main]
async fn main() -> ChatResult<()> {
    let address = env::args().nth(1).expect("Usage: server address");
    let listener = TcpListener::bind(address).await?;
    run(listener, signal::ctrl_c()).await;
    Ok(())
}

async fn run(listener: TcpListener, shutdown: impl Future) {
    tokio::select! {
        result = serve(listener) => {
            if let Err(error) = result {
                println!("failed to accept error: {}", error);
            }
        }
        _ = shutdown =>  {
            println!("shutting down");
        }
    }
}

async fn serve(listener: TcpListener) -> ChatResult<()> {
    let chat_group_table = Arc::new(GroupTable::new());
    loop {
        let (socket, addr) = listener.accept().await?;
        println!("accept:{}", addr);
        let groups = chat_group_table.clone();
        tokio::spawn(async move {
            let (sender, mut receiver) = mpsc::channel(100);
            let tx = Connection::run(socket, sender).await;
            while let Some(msg) = receiver.recv().await {
                let res = match msg {
                    FromClient::Join { group_name } => {
                        println!("client {} join {}", addr, group_name);
                        let group = groups.get_or_create(group_name);
                        group.join(tx.clone());
                        Ok(())
                    }
                    FromClient::Post {
                        group_name,
                        message,
                    } => match groups.get(&group_name) {
                        Some(group) => {
                            println!("client {} post {} in {}", addr, message, group_name);
                            group.post(message);
                            Ok(())
                        }
                        None => Err(format!("Group '{}' does not exist", group_name)),
                    },
                };
                if let Err(message) = res {
                    let report = FromServer::Error(message);
                    tx.send(report).await.unwrap();
                }
            }
        });
    }
}
