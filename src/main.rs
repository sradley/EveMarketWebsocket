//! # EveMarketAnalysis
//! 
//! ...
//! 
//! TODO: add logging

use std::sync::Arc;
use eve_market_analysis::handler::MarketHandler;

fn main() {
    ws::listen("127.0.0.1:3012", |out| {
        let handler = MarketHandler::new(
            vec![
                10000030,
            ],
            Arc::new(out)
        );
        handler.start();

        handler
    }).unwrap();
}
