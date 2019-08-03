//! # EveMarketAnalysis
//! 
//! ...
//! 
//! TODO: add wss support
//! TODO: write tests
//! TODO: write documentation
//! TODO: add AWS cognito authentication

use log::LevelFilter;
use eve_market_analysis::{run, Logger};

static LOGGER: Logger = Logger;

fn main() -> ws::Result<()> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .unwrap();

    let region_ids = vec![
        10000030,
        10000042,
    ];
    run(region_ids, "127.0.0.1", 3012)
}
