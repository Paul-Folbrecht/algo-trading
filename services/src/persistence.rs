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
    use domain::domain::RealizedPnL;
    use mongodb::bson::{self, doc, Bson};
    use serde::Serialize;
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
                                println!("Error writing object: {:?}", e);
                            }
                        },
                        Err(_) => {
                            println!("PersistenceService: Channel shut down");
                            break;
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
                let filter: bson::Document = doc! {
                    "symbol": order.symbol.clone(),
                    "date": order.date.format("%Y-%m-%d").to_string()
                };
                self.upsert("orders", order.id(), filter, &order)
            } else if let Some(position) = p.as_any().downcast_ref::<Position>() {
                let filter: bson::Document = doc! { "symbol": position.symbol.clone() };
                self.upsert("positions", position.id(), filter, &position)
            } else if let Some(pnl) = p.as_any().downcast_ref::<RealizedPnL>() {
                let filter: bson::Document = doc! { "id": pnl.id() };
                self.upsert("pnl", pnl.id(), filter, &pnl)
            } else {
                Err(format!("Cannot handle unknown type: {:?}", p.type_id()))
            }
        }

        fn upsert<T: ?Sized + Serialize>(
            &self,
            collection_name: &str,
            id: i64,
            filter: bson::Document,
            object: &T,
        ) -> Result<(), String> {
            let document: bson::Document = doc! {
                "$set": bson::to_bson(object).map_err(|e| e.to_string())?
            };
            let collection = self
                .client
                .database("algo-trading")
                .collection::<Bson>(collection_name);
            let options: mongodb::options::UpdateOptions =
                mongodb::options::UpdateOptions::builder()
                    .upsert(true)
                    .build();

            match collection.update_one(filter, document.to_owned(), options) {
                Ok(result) => {
                    let mongo_id = result
                        .upserted_id
                        .map(|id| id.as_object_id().expect("Cast to ObjectId failed"));
                    println!(
                        "Inserted/updated object with id, mongo id: {}, {:?}",
                        id, mongo_id
                    );
                }
                Err(e) => {
                    println!("Error inserting object: {:?}; {:?}", e, id);
                }
            };
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "./tests/persistence_test.rs"]
mod persistence_test;
