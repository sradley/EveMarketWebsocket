//! # EveMarketAnalysis
//! 
//! ...

// re-exports
pub mod handler;
pub mod worker;

use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use log::{Record, Metadata, info, warn, debug};
use chrono::Utc;
use ws::{Sender, Handler};
use ws::util::Token;

/// `Client` struct ...
pub struct Client {
    sender: Sender,
    channels: Mutex<HashSet<i32>>,
}

impl Client {
    /// `Client` constructor ...
    pub fn new(sender: Sender) -> Self {
        Self {
            sender,
            channels: Mutex::new(HashSet::new()),
        }
    }

    /// `subscribe` method ...
    pub fn subscribe(&self, channel: i32) {
        self.channels.lock().unwrap().insert(channel);
    }

    /// `unsubscribe` method ...
    pub fn unsubscribe(&self, channel: i32) {
        self.channels.lock().unwrap().remove(&channel);
    }

    /// `in_channel` method ...
    pub fn in_channel(&self, channel: i32) -> bool {
        self.channels.lock().unwrap().contains(&channel)
    }
}

/// `ClientList` type ...
type ClientList = Arc<Mutex<HashMap<Token, Arc<Client>>>>;

/// `Logger` struct ...
pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.target().contains(module_path!())
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let utc = Utc::now();
            println!("{} [{}]: {}", utc.format("%Y-%m-%d %T"), record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

/// `run` function ...
pub fn run(region_ids: Vec<i32>, ip: &str, port: i32) -> ws::Result<()> {
    info!("[websocket] listening on {}:{}", ip, port);
    let handler = Arc::new(handler::MarketHandler::new(
        region_ids,
        Arc::new(Mutex::new(HashMap::new())),
    ));

    handler.start();
    
    ws::listen(format!("{}:{}", ip, port), move |out| {
        SocketHandler::new(Arc::clone(handler.clients()), Arc::new(Client::new(out)))
    })
}

/// `SocketHandler` struct ...
pub struct SocketHandler {
    clients: ClientList,
    conn: Arc<Client>,
}

impl SocketHandler {
    /// `SocketHandler` constructor ...
    pub fn new(clients: ClientList, conn: Arc<Client>) -> Self {
        Self { clients, conn }
    }
}

impl Handler for SocketHandler {
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        info!(
            "[websocket] client ({}) connected from: {:?}",
            self.conn.sender.token().0,
            shake.peer_addr.unwrap()
        );

        self.conn.sender.send("Welcome to EveMarketAnalysis v0.1.0").unwrap();
        let mut clients = self.clients.lock().unwrap();
        clients.insert(
            self.conn.sender.token(),
            Arc::clone(&self.conn)
        );

        debug!("[websocket] clients connected `{}`", clients.len());

        Ok(())
    }

    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        info!(
            "[websocket] client ({}) disconnected ({:?}) {}",
            self.conn.sender.token().0,
            code,
            reason
        );

        let mut clients = self.clients.lock().unwrap();
        clients.remove(&self.conn.sender.token());

        debug!("[websocket] clients connected `{}`", clients.len());
    }

    fn on_error(&mut self, _: ws::Error) {
        warn!("[websocket] error occurred in websocket");
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        let msg = msg.into_text().unwrap();
        let msg: Vec<&str> = msg.split_whitespace().collect();

        if msg.len() >= 2 {
            match msg[0] {
                "subscribe" => {
                    let channel: i32 = match msg[1].parse() {
                        Ok(channel) => channel,
                        Err(_) => return Ok(()),
                    };

                    info!(
                        "[websocket] client ({}) subscribed to channel `{}`",
                        self.conn.sender.token().0,
                        channel,
                    );
                    self.conn.subscribe(channel);
                },
                "unsubscribe" => {
                    let channel: i32 = match msg[1].parse() {
                        Ok(channel) => channel,
                        Err(_) => return Ok(()),
                    };

                    info!(
                        "[websocket] client ({}) unsubscribed from channel `{}`",
                        self.conn.sender.token().0,
                        channel,
                    );

                    self.conn.unsubscribe(channel);
                },
                _ => (),
            }
        }

        Ok(())
    }
}