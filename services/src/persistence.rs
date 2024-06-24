use crossbeam_channel::{Receiver, Sender};
use domain::domain::{Order, Persistable, Position};
use mongodb::sync::Client;
use std::{
    sync::{atomic::AtomicBool, Arc},
    thread::JoinHandle,
};

pub trait PersistenceService {
    fn init(&self, shutdown: Arc<AtomicBool>) -> Result<JoinHandle<()>, String>;
    fn write(&self, p: Box<dyn Persistable + Send>) -> Result<(), String>;
    fn drop_positions(&self) -> Result<(), String>;
}

pub fn new(url: String) -> Arc<impl PersistenceService> {
    let client = Client::with_uri_str(url).expect("Could not connect to MongoDB");
    let (sender, receiver) = crossbeam_channel::unbounded();
    Arc::new(implementation::Persistence {
        client,
        sender,
        receiver,
    })
}

mod implementation {
    use super::*;
    use mongodb::bson::{self, Bson};
    use std::any::Any;

    pub struct Persistence {
        pub client: Client,
        pub sender: Sender<Box<dyn Persistable + Send>>,
        pub receiver: Receiver<Box<dyn Persistable + Send>>,
    }

    pub struct Writer {
        pub client: Client,
        pub receiver: Receiver<Box<dyn Persistable + Send>>,
    }

    impl PersistenceService for Persistence {
        fn init(&self, shutdown: Arc<AtomicBool>) -> Result<JoinHandle<()>, String> {
            let client = self.client.clone();
            let receiver = self.receiver.clone();
            let writer = Writer { client, receiver };

            let handle = std::thread::spawn(move || {
                while !shutdown.load(std::sync::atomic::Ordering::Relaxed) {
                    match writer.receiver.recv() {
                        Ok(p) => match writer.write(p) {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("Error writing object: {:?}", e);
                            }
                        },
                        Err(e) => {
                            eprintln!("Channel shut down: {:?}", e);
                        }
                    }
                }
            });
            Ok(handle)
        }

        fn write(&self, p: Box<dyn Persistable + Send>) -> Result<(), String> {
            self.sender.send(p).map_err(|e| e.to_string())
        }

        fn drop_positions(&self) -> Result<(), String> {
            self.client
                .database("algo-trading")
                .collection::<bson::Document>("positions")
                .drop(None)
                .map_err(|e| e.to_string())
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
                            println!("Inserted object with id, mongo id: {}, {}", id, mongo_id);
                        }
                        Err(e) => {
                            eprintln!("Error inserting object: {:?}; {:?}", e, id);
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
#[path = "./tests/persistence_test.rs"]
mod persistence_test;
