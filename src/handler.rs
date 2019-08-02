//! # MarketHandler
//! 
//! ...

use std::sync::{Arc, Mutex};
use ws::{Handler, Sender};
use log::{info, warn};
use crate::worker::WorkerHandler;

/// `SocketHandler` struct ...
pub struct SocketHandler;

impl Handler for SocketHandler {
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        info!(
            "[websocket] client connected from: {:?}",
            shake.peer_addr.unwrap()
        );
        Ok(())
    }

    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        info!("[websocket] client disconnected ({:?}) {}", code, reason);
    }

    fn on_error(&mut self, _: ws::Error) {
        warn!("[websocket] error occurred in websocket");
    }
}

/// `MarketHandler` struct ...
pub struct MarketHandler {
    workers: Vec<WorkerHandler>,
    clients: Arc<Mutex<Vec<Sender>>>,
}

impl MarketHandler {
    /// `MarketHandler` constructor ...
    pub fn new(region_ids: Vec<i32>, clients: Arc<Mutex<Vec<Sender>>>) -> Self {
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

    /// `add_client` method ...
    pub fn add_client(&self, client: Sender) {
        client.send("Welcome to EveMarketAnalysis v0.1.0").unwrap();
        let mut clients = self.clients.lock().unwrap();
        clients.push(client);
    }
}