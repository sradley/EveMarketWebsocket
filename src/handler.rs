//! # MarketHandler
//! 
//! ...

use std::sync::Arc;
use ws::{Handler, Sender};
use log::{info, warn, debug};
use crate::worker::WorkerHandler;
use crate::ClientList;

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

/// `MarketHandler` struct ...
pub struct MarketHandler {
    workers: Vec<WorkerHandler>,
    clients: ClientList,
}

impl MarketHandler {
    /// `MarketHandler` constructor ...
    pub fn new(region_ids: Vec<i32>, clients: ClientList) -> Self {
        let mut workers = vec![];
        for region_id in region_ids {
            workers.push(WorkerHandler::new(region_id, Arc::clone(&clients)));
        }

        Self {
            workers,
            clients,
        }
    }

    /// `start` method ...
    pub fn start(&self) {
        for worker in self.workers.iter() {
            worker.start();
        }
    }

    /// `stop` method ...
    pub fn stop(&self) {
        for worker in self.workers.iter() {
            worker.stop();
        }
    }

    /// `clients` method ...
    pub fn clients(&self) -> &ClientList {
        &self.clients
    }
}