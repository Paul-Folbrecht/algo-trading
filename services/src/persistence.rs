use crossbeam_channel::{Receiver, Sender};
use domain::domain::{Order, Persistable, Position};
use std::{
    sync::{atomic::AtomicBool, Arc},
    thread::JoinHandle,
};

pub trait PersistenceService {
    fn init(&self, shutdown: Arc<AtomicBool>) -> Result<JoinHandle<()>, String>;
    fn write(&self, p: Box<dyn Persistable + Send>) -> Result<(), String>;
    fn read_positions(&self) -> Result<Vec<Position>, String>;
}

pub fn new() -> Arc<impl PersistenceService> {
    let (sender, receiver) = crossbeam_channel::unbounded();
    Arc::new(implementation::Persistence { sender, receiver })
}

mod implementation {
    use core::serde::tradier_date_time_format;
    use std::any::Any;

    use super::*;
    use chrono::{DateTime, Local};
    use mongodb::{
        bson::{self, Bson},
        sync::Client,
    };
    use serde::Deserialize;

    pub struct Persistence {
        pub sender: Sender<Box<dyn Persistable + Send>>,
        pub receiver: Receiver<Box<dyn Persistable + Send>>,
    }

    pub struct Writer {
        pub client: Client,
        pub receiver: Receiver<Box<dyn Persistable + Send>>,
    }

    #[derive(Deserialize)]
    struct TradierPosiiton {
        pub id: i64,
        pub symbol: String,
        pub quantity: i64,
        pub cost_basis: f64,
        #[serde(with = "tradier_date_time_format")]
        pub date_acquired: DateTime<Local>,
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
                        Ok(p) => match writer.write(p) {
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

        fn write(&self, p: Box<dyn Persistable + Send>) -> Result<(), String> {
            self.sender.send(p).map_err(|e| e.to_string())
        }

        fn read_positions(&self) -> Result<Vec<Position>, String> {
            unimplemented!()
        }
    }

    impl Writer {
        fn write(&self, p: Box<dyn Persistable>) -> Result<(), String> {
            if let Some(order) = p.as_any().downcast_ref::<Order>() {
                let serialized = bson::to_bson(&order).map_err(|e| e.to_string())?;
                self.write_impl("orders", order.id(), &serialized)
            } else if let Some(position) = p.as_any().downcast_ref::<Position>() {
                let serialized = bson::to_bson(&position).map_err(|e| e.to_string())?;
                self.write_impl("positions", position.id(), &serialized)
            } else {
                Err(format!("Cannot handle unknown type: {:?}", p.type_id()))
            }
        }

        fn write_impl(&self, collection_name: &str, id: i64, object: &Bson) -> Result<(), String> {
            match object.as_document().map(|doc| doc.to_owned()) {
                Some(document) => {
                    let collection = self
                        .client
                        .database("algo-trading")
                        .collection(collection_name);
                    match collection.insert_one(document.to_owned(), None) {
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
