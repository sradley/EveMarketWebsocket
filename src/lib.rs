//! # EveMarketAnalysis
//! 
//! ...

// re-exports
pub mod handler;
pub mod worker;

use std::sync::{Arc, Mutex};
use log::{Record, Metadata, info};
use chrono::Utc;

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
        Arc::new(Mutex::new(vec![])),
    ));

    handler.start();
    
    ws::listen(format!("{}:{}", ip, port), move |out| {
        handler.add_client(out);
        handler::SocketHandler {}
    })
}