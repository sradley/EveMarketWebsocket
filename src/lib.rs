//! # EveMarketAnalysis
//! 
//! ...

// re-exports
pub mod handler;
pub mod worker;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use log::{Record, Metadata, info};
use chrono::Utc;
use ws::Sender;
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
        handler::SocketHandler::new(Arc::clone(handler.clients()), Arc::new(out))
    })
}