//! # WorkerHandler
//! 
//! ...

use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;
use chttp::HttpClient;
use serde_derive::{Deserialize, Serialize};
use ws::Sender;
use log::{info, warn};

/// `Order` struct ...
#[derive(Debug, Deserialize, Serialize)]
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
#[derive(Debug, Serialize)]
struct TypeOrders {
    sell_orders: Vec<Order>,
    buy_orders: Vec<Order>,
}

/// `RegionOrders` struct ...
#[derive(Debug, Serialize)]
struct RegionOrders {
    region_id: i32,
    orders: HashMap<i32, TypeOrders>,
}

/// `WorkerHandler` struct ...
pub struct WorkerHandler {
    region_id: i32,
    running: Arc<Mutex<bool>>,
    worker: Arc<Mutex<Worker>>,
    clients: Arc<Mutex<Vec<Sender>>>,
}

impl WorkerHandler {
    /// `WorkerHandler` constructor ...
    pub fn new(region_id: i32, clients: Arc<Mutex<Vec<Sender>>>) -> Self {
        Self {
            region_id,
            running: Arc::new(Mutex::new(true)),
            worker: Arc::new(Mutex::new(Worker::new(region_id))),
            clients,
        }
    }

    /// `start` method ...
    pub fn start(&self) {
        info!("[worker `{}`] started up", self.region_id);

        let running = Arc::clone(&self.running);
        let worker = Arc::clone(&self.worker);
        let clients = Arc::clone(&self.clients);

        thread::spawn(move || {
            let mut last_pull = Duration::from_secs(300);
            
            while *running.lock().unwrap() {
                if last_pull > Duration::from_secs(300) {
                    last_pull = Duration::from_secs(0);

                    let mut worker = worker.lock().unwrap();

                    let data = match (*worker).pull_data() {
                        Some(data) => data,
                        None => continue,
                    };

                    // convert the message to json and send via websocket
                    match serde_json::to_string(&data) {
                        Ok(data) => {
                            let clients = clients.lock().unwrap();

                            for client in (*clients).iter() {
                                match client.send(data.clone()) {
                                    Ok(_) => (),
                                    Err(_) => {
                                        warn!("[worker `{}`] can't send data", worker.region_id);
                                    },
                                }
                            }

                            info!("[worker `{}`] sent data", worker.region_id);
                        },
                        Err(_) => warn!("[worker `{}`] can't serialize json", worker.region_id),
                    };
                }

                last_pull += Duration::from_secs(1);
                thread::sleep(Duration::from_secs(1));
            }
        });
    }

    /// `stop` method ...
    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
        info!("[worker `{}`] stopped", self.region_id);
    }
}

/// `Worker` struct ...
struct Worker {
    region_id: i32,
    client: Arc<HttpClient>,
}

impl Worker {
    /// `Worker` constructor ...
    pub fn new(region_id: i32) -> Self {
        Self { region_id, client: Arc::new(HttpClient::new()) }
    }

    /// `pull_data` method ...
    pub fn pull_data(&mut self) -> Option<RegionOrders> {
        // one request to get num pages
        let (orders, pages) = self.get_pages();
        let pages = match pages {
            Some(pages) => pages,
            None => {
                warn!("[worker `{}`] can't get pages", self.region_id);
                return None
            }
        };

        if pages < 2 {
            return Some(self.parse_orders(orders))
        }

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
                    Err(_) => {
                        warn!("[worker `{}`] request failed", region_id);
                        return
                    },
                };

                let mut new_orders: Vec<Order> = res.body_mut().json()
                    .unwrap_or_else(|_| {
                        warn!("[worker `{}`] can't deserialize json", region_id);
                        vec![]
                    });

                (*orders.lock().unwrap()).append(&mut new_orders);
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap_or_else(|_| {
                warn!("[worker `{}`] could not join thread", self.region_id);
            });
        }

        // move orders out of mutex
        let orders = Arc::try_unwrap(orders_mutex)
            .unwrap_or_else(|_| {
                warn!("[worker `{}`] mutex still has multiple owners", self.region_id);
                Mutex::new(vec![])
            });
        let orders = orders.into_inner()
            .unwrap_or_else(|_| {
                warn!("[worker `{}`] can't unlock mutex", self.region_id);
                vec![]
            });

        Some(self.parse_orders(orders))
    }

    // `get_pages` method ...
    fn get_pages(&mut self) -> (Vec<Order>, Option<i32>) {
        let uri = format!(
            "https://esi.evetech.net/latest/markets/{}/orders/\
            ?datasource=tranquility&order_type=all&page=1",
            self.region_id
        );

        let mut res = match (*self.client).get(uri) {
            Ok(res) => res,
            Err(_) => {
                warn!("[worker `{}`] request failed", self.region_id);
                return (vec![], None)
            },
        };

        let pages = match res.headers().get("x-pages") {
            Some(pages) => pages,
            None => return (vec![], None),
        };
        let pages = match pages.to_str() {
            Ok(pages) => pages,
            Err(_) => return (vec![], None),
        };
        let pages = match pages.parse() {
            Ok(pages) => pages,
            Err(_) => return (vec![], None),
        };

        (
            res.body_mut().json()
                .unwrap_or_else(|_| {
                    warn!("[worker `{}`] can't deserialize json", self.region_id);
                    vec![]
                }),
            Some(pages)
        )
    }

    /// `parse_orders` method ...
    fn parse_orders(&mut self, orders: Vec<Order>) -> RegionOrders {
        let mut type_orders: HashMap<i32, TypeOrders> = HashMap::new();

        for order in orders {
            type_orders.entry(order.type_id).or_insert(TypeOrders {
                sell_orders: vec![],
                buy_orders: vec![],
            });

            // these unwraps are okay, as the above statement ensures that the
            // element exists in the hashmap
            if order.is_buy_order {
                type_orders.get_mut(&order.type_id).unwrap().buy_orders
                    .push(order);
            } else {
                type_orders.get_mut(&order.type_id).unwrap().sell_orders
                    .push(order);
            }
        }

        RegionOrders { region_id: self.region_id, orders: type_orders }
    }
}