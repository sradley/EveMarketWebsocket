//! # MarketHandler
//! 
//! ...

use crate::worker::WorkerHandler;

/// `MarketHandler` struct ...
pub struct MarketHandler {
    workers: Vec<WorkerHandler>
}

impl MarketHandler {
    /// `MarketHandler` constructor ...
    pub fn new(region_ids: Vec<i32>) -> Self {
        let workers = region_ids.iter()
            .map(|region_id| {
                WorkerHandler::new(*region_id)
            })
            .collect();

        Self {
            workers,
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