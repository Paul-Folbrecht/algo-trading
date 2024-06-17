use domain::domain::Order;
use mongodb::{options::ClientOptions, Client};
use std::sync::Arc;

pub trait PersistenceService {
    fn write_order(&self, order: Order) -> Result<(), String>;
}

pub async fn new() -> Arc<impl PersistenceService> {
    let client_uri = "mongodb://127.0.0.1:27017";
    let options = ClientOptions::parse(&client_uri).await.unwrap();
    let client = Client::with_options(options).unwrap();
    Arc::new(implementation::Persistence { client })
}

mod implementation {
    use mongodb::{
        bson::{self},
        Client,
    };

    use super::*;

    pub struct Persistence {
        pub client: Client,
    }

    impl PersistenceService for Persistence {
        fn write_order(&self, _order: Order) -> Result<(), String> {
            let order = _order.clone();
            let serialized = bson::to_bson(&order).map_err(|e| e.to_string())?;
            match serialized.as_document().map(|doc| doc.to_owned()) {
                Some(document) => {
                    let orders = self.client.database("oms").collection("orders");
                    let id = order.tradier_id.unwrap();
                    tokio::spawn(async move {
                        match orders.insert_one(document.to_owned(), None).await {
                            Ok(insert_result) => {
                                let mongo_id = insert_result
                                    .inserted_id
                                    .as_object_id()
                                    .expect("Retrieved _id should have been of type ObjectId");
                                println!("Inserted order with id, mongo id: {}, {}", id, mongo_id);
                            }
                            Err(e) => {
                                eprintln!("Error inserting order: {:?}; {:?}", e, id);
                            }
                        }
                    });
                    Ok(())
                }
                None => Err("Could not serialize order".to_string()),
            }
        }

        // fn write_order(&self, _order: Order) -> Result<(), String> {
        //     let order = _order.clone();
        //     let serialized = bson::to_bson(&order).map_err(|e| e.to_string())?;
        //     let document = serialized.as_document().unwrap();
        //     let orders = self.client.database("oms").collection("orders");
        //     let id = order.tradier_id.unwrap();
        //     let result: JoinHandle<()> = tokio::spawn(async move {
        //         match orders.insert_one(document.to_owned(), None).await {
        //             Ok(insert_result) => {
        //                 let id = insert_result
        //                     .inserted_id
        //                     .as_object_id()
        //                     .expect("Retrieved _id should have been of type ObjectId");
        //                 println!("Inserted order with id: {}", id);
        //             }
        //             Err(e) => {
        //                 //eprintln!("Error inserting order: {:?}; {:?}", e, id);
        //             }
        //         }
        //     });
        //     Ok(())
        // }
    }
}
