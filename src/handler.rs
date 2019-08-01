//! # MarketHandler
//! 
//! ...

use std::sync::Arc;
use ws::{Handler, Sender};
use crate::worker::WorkerHandler;

/// `MarketHandler` struct ...
pub struct MarketHandler {
    workers: Vec<WorkerHandler>,
    //socket: Arc<Sender>,
}

impl MarketHandler {
    /// `MarketHandler` constructor ...
    pub fn new(region_ids: Vec<i32>, socket: Arc<Sender>) -> Self {
        let mut workers = vec![];
        for region_id in region_ids {
            workers.push(WorkerHandler::new(region_id, Arc::clone(&socket)));
        }

        Self {
            workers,
            //socket,
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
}

impl Handler for MarketHandler {}