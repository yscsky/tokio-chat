use std::{char, env, sync::Arc};

use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
};
use tokio_chat::{conn::Connection, ChatResult, FromClient, FromServer};

#[tokio::main]
async fn main() -> ChatResult<()> {
    let address = env::args().nth(1).expect("Usage: server address");
    let socket = TcpStream::connect(address).await?;
    let (sender, receiver) = mpsc::channel(100);
    let tx = Connection::run(socket, sender).await;

    let mut cmd_task = tokio::spawn(async {
        send_command(tx).await;
    });

    let mut reply_task = tokio::spawn(async {
        handle_replies(receiver).await;
    });

    if tokio::try_join!(&mut cmd_task, &mut reply_task).is_err() {
        cmd_task.abort();
        reply_task.abort();
    }
    Ok(())
}

async fn send_command(tx: Sender<FromClient>) {
    println!(
        "Commands:\n\
join GROUP\n\
post GROUP MESSAGE...\n\
Type Control-D (on Unix) or Control-Z (on Windows) \
to close the connection."
    );
    let mut cmd_lines = BufReader::new(io::stdin()).lines();
    while let Ok(cmd_result) = cmd_lines.next_line().await {
        let cmd = cmd_result.unwrap();
        let request = match parse_command(&cmd) {
            Some(request) => request,
            None => continue,
        };
        tx.send(request).await.unwrap();
    }
}

fn parse_command(line: &str) -> Option<FromClient> {
    let (command, rest) = get_next_token(line)?;
    match command {
        "post" => {
            let (group, rest) = get_next_token(rest)?;
            let message = rest.trim_start().to_string();
            return Some(FromClient::Post {
                group_name: Arc::new(group.into()),
                message: Arc::new(message),
            });
        }
        "join" => {
            let (group, rest) = get_next_token(rest)?;
            if !rest.trim_start().is_empty() {
                return None;
            }
            return Some(FromClient::Join {
                group_name: Arc::new(group.into()),
            });
        }
        _ => {
            eprintln!("Unrecognized command: {:?}", line);
            None
        }
    }
}

fn get_next_token(mut input: &str) -> Option<(&str, &str)> {
    input = input.trim_start();
    if input.is_empty() {
        return None;
    }
    match input.find(char::is_whitespace) {
        Some(space) => Some((&input[0..space], &input[space..])),
        None => Some((input, "")),
    }
}

async fn handle_replies(mut receiver: Receiver<FromServer>) {
    while let Some(msg) = receiver.recv().await {
        match msg {
            FromServer::Message {
                group_name,
                message,
            } => {
                println!("message posted to {}: {}", group_name, message);
            }
            FromServer::Error(message) => {
                println!("error from server: {}", message);
            }
        }
    }
}
