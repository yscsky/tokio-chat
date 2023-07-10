use std::{env, future::Future, sync::Arc};

use conn::Connection;
use group::GroupTable;
use tokio::{net::TcpListener, signal};
use tokio_chat::{ChatResult, FromClient, FromServer};

mod conn;
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
        let (socket, _) = listener.accept().await?;
        let groups = chat_group_table.clone();
        tokio::spawn(async {
            Connection::run(socket, move |msg, tx| {
                let res = match msg {
                    FromClient::Join { group_name } => {
                        println!("client join {}", group_name);
                        let group = groups.get_or_create(group_name);
                        group.join(tx.clone());
                        Ok(())
                    }
                    FromClient::Post {
                        group_name,
                        message,
                    } => match groups.get(&group_name) {
                        Some(group) => {
                            println!("client post {} in {}", message, group_name);
                            group.post(message);
                            Ok(())
                        }
                        None => Err(format!("Group '{}' does not exist", group_name)),
                    },
                };
                if let Err(message) = res {
                    let report = FromServer::Error(message);
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        tx.send(report).await.unwrap();
                    });
                }
            });
        });
    }
}
