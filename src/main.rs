
use crate::core::leases::LeaseV4;
use crate::core::packet_context::HardwareAddress;
use ::core::sync;
use std::{sync::{Mutex, Arc}, net::Ipv4Addr, time::Duration};

use chrono::{Utc};
use tokio::time::sleep;
use utils::data::{DbManager, RuntimeStorage, DataPool};

pub mod core;
pub mod hooks;
pub mod utils;


#[tokio::main]
async fn main() {
    let db = DbManager::new(String::from("dhcp"), String::from("frozenpeach"), String::from("poney"), String::from("127.0.0.1:3333"));
    let storage = RuntimeStorage::new(Arc::new(Mutex::new(db)));
    let manager = Arc::new(Mutex::new(storage));

    let storage = manager.clone(); 
    let lease4_pool:DataPool<LeaseV4> = DataPool::new(String::from("lease4"), String::from("(id INT, ip_address VARCHAR(255), hardware_address VARCHAR(255), expiration VARCHAR(255) )"));
    let lease6_pool:DataPool<LeaseV4> = DataPool::new(String::from("lease6"), String::from("(id INT, ip_address VARCHAR(255), hardware_address VARCHAR(255), expiration VARCHAR(255) )"));
    let lease4 = LeaseV4{
        ip_address : Ipv4Addr::BROADCAST,
        expiration : Utc::now(),
        hardware_address : HardwareAddress::new([0;16]),
        id : 10
    };
    let lease6 = LeaseV4{
        ip_address : Ipv4Addr::BROADCAST,
        expiration : Utc::now(),
        hardware_address : HardwareAddress::new([0;16]),
        id : 10
    };

    let synchro = manager.clone();
    tokio::spawn(async move {
        let mut storage = storage.lock().unwrap();
        storage.add_pool(lease4_pool);
        storage.add_pool(lease6_pool);
        storage.store(lease4, String::from("lease4")).unwrap();
        storage.store(lease6, String::from("lease6")).unwrap();
    });
    
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(1)).await;  
            synchro.lock().unwrap().sync();
        }
    }).await.unwrap();

    
    


}


