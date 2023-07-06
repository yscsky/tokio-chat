use std::{
    env,
    sync::{Arc, Mutex},
};

use group::GroupTable;
use tokio::net::{TcpListener, TcpStream};
use tokio_chat::{
    utils::{self, ChatResult},
    FromServer,
};

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
        tokio::spawn(async { log_error(serve(socket, groups).await) });
    }
}

fn log_error(result: ChatResult<()>) {
    if let Err(error) = result {
        eprintln!("Error: {}", error);
    }
}

async fn serve(socket: TcpStream, groups: Arc<GroupTable>) -> ChatResult<()> {
    Ok(())
}

struct OutBound(Mutex<TcpStream>);

impl OutBound {
    fn new(to_client: TcpStream) -> OutBound {
        OutBound(Mutex::new(to_client))
    }

    async fn send(&self, packet: FromServer) -> ChatResult<()> {
        let mut guard = self.0.lock().unwrap();
        utils::send_as_json(&mut *guard, &packet).await?;
        Ok(())
    }
}
