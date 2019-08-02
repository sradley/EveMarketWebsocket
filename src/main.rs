//! # EveMarketAnalysis
//! 
//! ...
//! 
//! TODO: add channels
//! TODO: add wss support
//! TODO: write tests
//! TODO: write documentation

use log::LevelFilter;
use eve_market_analysis::{run, Logger};

static LOGGER: Logger = Logger;

fn main() -> ws::Result<()> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .unwrap();

    run("127.0.0.1", 3012)
}
