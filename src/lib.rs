//! # EveMarketAnalysis
//! 
//! ...

// re-exports
pub mod handler;
pub mod worker;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use log::{Record, Metadata, info, warn, debug};
use chrono::Utc;
use ws::{Sender, Handler};
use ws::util::Token;

/// `ClientList` type ...
type ClientList = Arc<Mutex<HashMap<Token, Arc<Sender>>>>;

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
        SocketHandler::new(Arc::clone(handler.clients()), Arc::new(out))
    })
}

/// `SocketHandler` struct ...
pub struct SocketHandler {
    clients: ClientList,
    conn: Arc<Sender>,
}

impl SocketHandler {
    /// `SocketHandler` constructor ...
    pub fn new(clients: ClientList, conn: Arc<Sender>) -> Self {
        Self { clients, conn }
    }
}

impl Handler for SocketHandler {
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        info!("[websocket] client connected from: {:?}", shake.peer_addr.unwrap());

        self.conn.send("Welcome to EveMarketAnalysis v0.1.0").unwrap();
        let mut clients = self.clients.lock().unwrap();
        clients.insert(self.conn.token(), Arc::clone(&self.conn));

        debug!("[websocket] clients connected `{}`", clients.len());

        Ok(())
    }

    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        info!("[websocket] client disconnected ({:?}) {}", code, reason);

        let mut clients = self.clients.lock().unwrap();
        clients.remove(&self.conn.token());

        debug!("[websocket] clients connected `{}`", clients.len());
    }

    fn on_error(&mut self, _: ws::Error) {
        warn!("[websocket] error occurred in websocket");
    }
}