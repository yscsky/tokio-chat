use std::{env, sync::Arc};

use tokio::net::TcpStream;
use tokio_chat::{conn::Connection, ChatResult, FromClient};

#[tokio::main]
async fn main() -> ChatResult<()> {
    let address = env::args().nth(1).expect("Usage: server address");
    let socket = TcpStream::connect(address).await?;
    let mut conn = Connection::new(socket);
    let join_cmd = FromClient::Join {
        group_name: Arc::new("group1".into()),
    };
    conn.send(&join_cmd).await?;

    let post_cmd = FromClient::Post {
        group_name: Arc::new("group1".into()),
        message: Arc::new("hello".into()),
    };
    conn.send(&post_cmd).await?;
    Ok(())
}
