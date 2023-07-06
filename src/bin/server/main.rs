use std::{env, sync::Arc};

use group::GroupTable;
use tokio::net::TcpListener;
use tokio_chat::utils::ChatResult;

mod group;

#[tokio::main]
async fn main() -> ChatResult<()> {
    let address = env::args().nth(1).expect("Usage: server address");
    let chat_group_table = Arc::new(GroupTable::new());

    let listener = TcpListener::bind(address).await?;
    loop {
        let (socket, addr) = listener.accept().await?;
        println!("accept socket: {}", addr);
        let groups = chat_group_table.clone();
        tokio::spawn(async {});
    }
}

fn log_error(result: ChatResult<()>) {
    if let Err(error) = result {
        eprintln!("Error: {}", error);
    }
}
