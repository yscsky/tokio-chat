use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio::sync::{
    broadcast::{self, error::RecvError},
    mpsc::Sender,
};
use tokio_chat::FromServer;

pub struct GroupTable(Mutex<HashMap<Arc<String>, Arc<Group>>>);

pub struct Group {
    name: Arc<String>,
    sender: broadcast::Sender<Arc<String>>,
}

impl GroupTable {
    pub fn new() -> GroupTable {
        GroupTable(Mutex::new(HashMap::new()))
    }

    pub fn get(&self, name: &String) -> Option<Arc<Group>> {
        self.0.lock().unwrap().get(name).cloned()
    }

    pub fn get_or_create(&self, name: Arc<String>) -> Arc<Group> {
        self.0
            .lock()
            .unwrap()
            .entry(name.clone())
            .or_insert(Arc::new(Group::new(name)))
            .clone()
    }
}

impl Group {
    pub fn new(name: Arc<String>) -> Group {
        let (sender, _) = broadcast::channel(1000);
        Group { name, sender }
    }

    pub fn join(&self, tx: Sender<FromServer>) {
        let receiver = self.sender.subscribe();
        tokio::spawn(handle_subscribler(self.name.clone(), receiver, tx));
    }

    pub fn post(&self, message: Arc<String>) {
        let _ = self.sender.send(message);
    }
}

async fn handle_subscribler(
    group_name: Arc<String>,
    mut receiver: broadcast::Receiver<Arc<String>>,
    tx: Sender<FromServer>,
) {
    loop {
        let packet = match receiver.recv().await {
            Ok(message) => FromServer::Message {
                group_name: group_name.clone(),
                message: message.clone(),
            },
            Err(RecvError::Lagged(n)) => {
                FromServer::Error(format!("Dropped {n} messages from {group_name}"))
            }
            Err(RecvError::Closed) => break,
        };
        if tx.send(packet).await.is_err() {
            break;
        }
    }
}
