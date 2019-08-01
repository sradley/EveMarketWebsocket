//! # WorkerHandler
//! 
//! ...

use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;

use chttp::HttpClient;
use serde_derive::Deserialize;

/// `WorkerHandler` struct ...
pub struct WorkerHandler {
    id: i32,
    running: Arc<Mutex<bool>>,
    worker: Arc<Mutex<Worker>>,
}

impl WorkerHandler {
    /// `WorkerHandler` constructor ...
    pub fn new(region_id: i32) -> Self {
        Self {
            id: region_id,
            running: Arc::new(Mutex::new(false)),
            worker: Arc::new(Mutex::new(Worker::new(region_id))),
        }
    }

    /// `start` method ...
    pub fn start(&self) {
        *self.running.lock().unwrap() = true;
        let running = Arc::clone(&self.running);
        let worker = Arc::clone(&self.worker);

        let handle = thread::spawn(move || {
            let mut last_pull = Duration::from_secs(0);
            
            while *running.lock().unwrap() {
                if last_pull > Duration::from_secs(300) {
                    (*worker.lock().unwrap()).pull_data();

                    last_pull = Duration::from_secs(0);
                }

                last_pull += Duration::from_secs(1);
                thread::sleep(Duration::from_secs(1));
            }
        });

        println!("Worker [{}] shutting down...", self.id);
        handle.join()
            .expect("error: unable to join worker thread");
    }

    /// `stop` method ...
    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }
}

/// `Worker` struct ...
struct Worker {
    region_id: i32,
    client: Arc<HttpClient>,
    orders: HashMap<i32, TypeOrders>,
}

impl Worker {
    /// `Worker` constructor ...
    pub fn new(region_id: i32) -> Self {
        Self {
            region_id,
            client: Arc::new(HttpClient::new()),
            orders: HashMap::new(),
        }
    }

    /// `pull_data` method ...
    pub fn pull_data(&self) {
        // one request to get num pages
        let (orders, pages) = self.get_pages();
        let orders_mutex = Arc::new(Mutex::new(orders));

        // rest of the requests
        for i in 2..(pages+1) {
            let client = Arc::clone(&self.client);
            let orders = Arc::clone(&orders_mutex);

            thread::spawn(move || {
                let uri = format!("{}", i);

                let mut res = match (*client).get(uri) {
                    Ok(res) => res,
                    Err(err) => {
                        if cfg!(debug_assertions) {
                            eprintln!("error: request failed: {}", err);
                        }
                        return
                    },
                };

                let mut new_orders: Vec<Order> = res.body_mut().json()
                    .unwrap_or_else(|_| vec![]);

                if cfg!(debug_assertions) && new_orders.is_empty() {
                    eprintln!("error: unable to deserialize json");
                }

                (*orders.lock().unwrap()).append(&mut new_orders);
            });
        }

        self.parse_orders();

        println!("Worker [{}] pulled data.", self.region_id);
    }

    // `get_pages` method ...
    fn get_pages(&self) -> (Vec<Order>, i32) {
        let uri = "";

        let mut res = match (*self.client).get(uri) {
            Ok(res) => res,
            Err(err) => {
                if cfg!(debug_assertions) {
                    eprintln!("error: request failed: {}", err);
                }
                return (vec![], -1)
            },
        };

        // should handle these unwraps
        let pages: i32 = res.headers().get("x-pages").unwrap()
            .to_str().unwrap()
            .parse().unwrap();

        let orders: Vec<Order> = res.body_mut().json()
            .unwrap_or_else(|_| vec![]);

        if cfg!(debug_assertions) && orders.is_empty() {
            eprintln!("error: unable to deserialize json");
        }

        (orders, pages)
    }

    /// `parse_orders` method ...
    fn parse_orders(&self) {

    }
}

/// `Order` struct ...
#[derive(Debug, Deserialize)]
struct Order {

}

/// `TypeOrders` struct ...
#[derive(Debug)]
struct TypeOrders {
    sell_orders: Vec<Order>,
    buy_orders: Vec<Order>,
}