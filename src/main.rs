//! # EveMarketAnalysis
//! 
//! ...

use std::thread;
use std::time::Duration;
use eve_market_analysis::handler::MarketHandler;

fn main() {
    let handle = MarketHandler::new(
        vec![
            10000030,
            10000032,
        ]
    );

    handle.start();

    thread::sleep(Duration::from_secs(500));

    handle.stop();
}
