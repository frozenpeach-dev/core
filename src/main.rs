pub mod core;
pub mod hooks;
pub mod utils;
use std::{sync::{Mutex, Arc}, time::Duration, net::Ipv4Addr};


use chrono::Utc;
use mysql::prelude::FromRow;
use tokio::{self, time::sleep};
use utils::data::DbManager;
use crate::{core::packet_context::HardwareAddress, utils::data::Data};

use crate::{utils::data::RuntimeStorage, core::leases::LeaseV4};

#[tokio::main]
async fn main() {
    let db = Arc::new(Mutex::new(DbManager::new(String::from("dhcp"), String::from("frozenpeach"), String::from("poney"), String::from("127.0.0.1:3333"))));
    
    let sto = RuntimeStorage::new(db, String::from("lease"));
    let sto = Arc::new(Mutex::new(sto));
    fn test (k : &u64, v : &mut LeaseV4) -> bool{
        v.expiration > Utc::now()
    }
    let test = test as fn(&u64, &mut LeaseV4) -> bool;
    let synchronizer = sto.clone();

    tokio::spawn(async move {
        loop{
            println!("syncing");
            synchronizer.lock().unwrap().sync();
            sleep(Duration::from_secs(1)).await
        }
        
    });
    let sto = sto.clone();
    
    for i in 1..100 {
        let lease = LeaseV4{
            ip_address : Ipv4Addr::BROADCAST,
            expiration : Utc::now(),
            hardware_address : HardwareAddress::new([0u8; 16]),
            mysql_table : String::from("lease"),
            id : i
        };
        sto.lock().unwrap().store(lease).unwrap();

    }


    sleep(Duration::from_secs(10)).await;
    fn test2 (k : &u64, v : &mut LeaseV4) -> bool{
        v.expiration > Utc::now()
    }
    sto.lock().unwrap().add_filter(test2);


    let lease = LeaseV4{
        ip_address : Ipv4Addr::BROADCAST,
        expiration : Utc::now(),
        hardware_address : HardwareAddress::new([0u8; 16]),
        mysql_table : String::from("lease"),
        id : 9
    };

    sleep(Duration::from_secs(100)).await;

}


