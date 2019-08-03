//! # MarketHandler
//! 
//! ...

use std::sync::Arc;
use crate::worker::WorkerHandler;
use crate::ClientList;

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