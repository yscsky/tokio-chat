use std::{env, future::Future, sync::Arc};

use group::GroupTable;
use tokio::{net::TcpListener, signal};
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
        let (socket, _) = listener.accept().await?;
        let mut conn = Connection::new(socket);
        let groups = chat_group_table.clone();
        tokio::spawn(async move {
            if let Err(error) = handle(&mut conn, groups).await {
                println!("failed to handle error: {}", error);
            }
        });
    }
}

async fn handle(conn: &mut Connection, groups: Arc<GroupTable>) -> ChatResult<()> {
    loop {
        println!("is block here");
        let from_client: FromClient = conn.receive().await?;
        println!("from: {:?}", from_client);
        let res = match from_client {
            FromClient::Join { group_name } => {
                let group = groups.get_or_create(group_name);
                group.join(conn);
                Ok(())
            }
            FromClient::Post {
                group_name,
                message,
            } => match groups.get(&group_name) {
                Some(group) => {
                    group.post(message);
                    Ok(())
                }
                None => Err(format!("Group '{}' does not exist", group_name)),
            },
        };
        if let Err(message) = res {
            let report = FromServer::Error(message);
            conn.send(&report).await?;
        }
    }
}
