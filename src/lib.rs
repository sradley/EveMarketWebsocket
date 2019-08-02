//! # EveMarketAnalysis
//! 
//! ...

// re-exports
pub mod handler;
pub mod worker;

use std::sync::Arc;
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
            println!(
                "{} [{}]: {}",
                utc.format("%Y-%m-%d %T"),
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

/// `run` function ...
pub fn run(ip: &str, port: i32) -> ws::Result<()> {
    info!("[websocket] listening on {}:{}", ip, port);
    
    ws::listen(format!("{}:{}", ip, port), |out| {
        let handler = handler::MarketHandler::new(
            vec![
                10000030,
            ],
            Arc::new(out)
        );
        handler.start();

        handler
    })
}