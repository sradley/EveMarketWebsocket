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
        println!("Worker [{}] starting up...", self.id);

        *self.running.lock().unwrap() = true;
        let running = Arc::clone(&self.running);
        let worker = Arc::clone(&self.worker);

        thread::spawn(move || {
            let mut last_pull = Duration::from_secs(300);
            
            while *running.lock().unwrap() {
                if last_pull > Duration::from_secs(300) {
                    (*worker.lock().unwrap()).pull_data();

                    last_pull = Duration::from_secs(0);
                }

                last_pull += Duration::from_secs(1);
                thread::sleep(Duration::from_secs(1));
            }
        });
    }

    /// `stop` method ...
    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
        println!("Worker [{}] shutting down...", self.id);
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
    pub fn pull_data(&mut self) {
        // one request to get num pages
        let (orders, pages) = self.get_pages();

        let orders_mutex = Arc::new(Mutex::new(orders));
        let mut handles = vec![];
        let region_id = self.region_id;

        // rest of the requests
        for i in 2..(pages+1) {
            let client = Arc::clone(&self.client);
            let orders = Arc::clone(&orders_mutex);

            let handle = thread::spawn(move || {
                let uri = format!(
                    "https://esi.evetech.net/latest/markets/{}/orders/\
                    ?datasource=tranquility&order_type=all&page={}",
                    region_id,
                    i,
                );

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

                (*orders.lock().unwrap()).append(&mut new_orders);
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // move orders out of mutex
        let orders = Arc::try_unwrap(orders_mutex)
            .expect("error: mutex still has multiple owners");
        let orders = orders.into_inner()
            .expect("error: unable to unlock mutex");
        self.parse_orders(orders);
    }

    // `get_pages` method ...
    fn get_pages(&mut self) -> (Vec<Order>, i32) {
        let uri = format!(
            "https://esi.evetech.net/latest/markets/{}/orders/\
            ?datasource=tranquility&order_type=all&page=1",
            self.region_id
        );

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

        (res.body_mut().json().unwrap_or_else(|_| vec![]), pages)
    }

    /// `parse_orders` method ...
    fn parse_orders(&mut self, orders: Vec<Order>) {
        let mut type_orders: HashMap<i32, TypeOrders> = HashMap::new();

        for order in orders {
            type_orders.entry(order.type_id).or_insert(TypeOrders {
                sell_orders: vec![],
                buy_orders: vec![],
            });

            if order.is_buy_order {
                type_orders.get_mut(&order.type_id).unwrap().buy_orders
                    .push(order);
            } else {
                type_orders.get_mut(&order.type_id).unwrap().sell_orders
                    .push(order);
            }
        }

        self.orders = type_orders;
    }
}

/// `Order` struct ...
#[derive(Debug, Deserialize)]
struct Order {
    duration: i32,
    is_buy_order: bool,
    issued: String,
    location_id: i64,
    price: f64,
    system_id: i32,
    type_id: i32,
    volume_remain: i32,
    volume_total: i32,
}

/// `TypeOrders` struct ...
#[derive(Debug)]
struct TypeOrders {
    sell_orders: Vec<Order>,
    buy_orders: Vec<Order>,
}