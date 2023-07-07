use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio::sync::broadcast;
use tokio_chat::conn::Connection;

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
        Group {
            name: name,
            sender: sender,
        }
    }

    pub fn join(&self, conn: &mut Connection) {
        let receiver = self.sender.subscribe();
        // tokio::spawn(handle_subscribler(self.name.clone(), receiver, conn));
    }

    pub fn post(&self, message: Arc<String>) {
        let _ = self.sender.send(message);
    }
}

async fn handle_subscribler(
    group_name: Arc<String>,
    mut receiver: broadcast::Receiver<Arc<String>>,
    conn: &mut Connection,
) {
}
