use crossbeam_channel::{Receiver, Sender};
use domain::domain::{Order, Persistable, Position};
use std::{
    sync::{atomic::AtomicBool, Arc},
    thread::JoinHandle,
};

pub trait PersistenceService {
    fn init(&self, shutdown: Arc<AtomicBool>) -> Result<JoinHandle<()>, String>;
    fn write_order(&self, order: Order) -> Result<(), String>;
    fn write_position(&self, position: Position) -> Result<(), String>;
    fn read_positions(&self) -> Result<Vec<Position>, String>;
}

pub fn new() -> Arc<impl PersistenceService> {
    let (sender, receiver) = crossbeam_channel::unbounded();
    Arc::new(implementation::Persistence { sender, receiver })
}

mod implementation {
    use std::any::Any;

    use super::*;
    use mongodb::{
        bson::{self},
        sync::Client,
    };

    pub struct Persistence {
        pub sender: Sender<Order>,
        pub receiver: Receiver<Order>,
    }

    pub struct Writer {
        pub client: Client,
        pub receiver: Receiver<Order>,
    }

    impl PersistenceService for Persistence {
        fn init(&self, shutdown: Arc<AtomicBool>) -> Result<JoinHandle<()>, String> {
            let receiver = self.receiver.clone();
            let handle = std::thread::spawn(move || {
                let uri = "mongodb://127.0.0.1:27017";
                let client = Client::with_uri_str(uri).expect("Could not connect to MongoDB");
                let writer = Writer { client, receiver };

                while !shutdown.load(std::sync::atomic::Ordering::Relaxed) {
                    match writer.receiver.recv() {
                        Ok(order) => match writer.write_order(&order) {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("Error writing order: {:?}", e);
                            }
                        },
                        Err(e) => {
                            eprintln!("Error receiving orders: {:?}", e);
                        }
                    }
                }
            });
            Ok(handle)
        }

        fn write_order(&self, order: Order) -> Result<(), String> {
            self.sender.send(order.clone()).map_err(|e| e.to_string())
        }

        fn write_position(&self, position: Position) -> Result<(), String> {
            unimplemented!()
        }

        fn read_positions(&self) -> Result<Vec<Position>, String> {
            unimplemented!()
        }
    }

    impl Writer {
        fn write(&self, p: Box<dyn Persistable>) -> Result<(), String> {
            let _: &Order = match p.as_any().downcast_ref::<Order>() {
                Some(b) => b,
                None => panic!("&a isn't a B!"),
            };
        }

        fn write_order(&self, order: &Order) -> Result<(), String> {
            let serialized = bson::to_bson(&order).map_err(|e| e.to_string())?;
            match serialized.as_document().map(|doc| doc.to_owned()) {
                Some(document) => {
                    let orders = self.client.database("algo-trading").collection("orders");
                    let id = order.tradier_id.unwrap();
                    match orders.insert_one(document.to_owned(), None) {
                        Ok(insert_result) => {
                            let mongo_id = insert_result.inserted_id.as_object_id().expect(
                                format!("Cast to ObjectId failed; order id: {:?}", id).as_str(),
                            );
                            println!(
                                "Inserted order with order id, mongo id: {}, {}",
                                id, mongo_id
                            );
                        }
                        Err(e) => {
                            eprintln!("Error inserting order: {:?}; {:?}", e, id);
                        }
                    };
                    Ok(())
                }
                None => Err("Could not serialize order: ".to_string()),
            }
        }
    }
}

#[cfg(test)]
#[path = "./tests/persistence.rs"]
mod persistence;
