pub mod core;
pub mod hooks;
pub mod utils;
use std::{sync::{Mutex, Arc}, time::Duration, net::Ipv4Addr};


use chrono::{DateTime, NaiveTime, Local, Utc};
use mysql::params;
use tokio::{self, time::sleep};

use utils::data::DbManager;
use crate::core::packet_context::HardwareAddress;

use crate::{utils::data::RuntimeStorage, core::leases::LeaseV4};

#[tokio::main]
async fn main() {
    let db = Arc::new(Mutex::new(DbManager::new(String::from("dhcp"), String::from("frozenpeach"), String::from("poney"), String::from("127.0.0.1:3333"))));
    let database = db.clone();
    let bob =tokio::spawn(async move {
        let db = database.lock().unwrap();
        //let result : Vec<String> = db.exec_and_return(String::from("INSERT INTO lease VALUE (:address, :name, :id)"), params! {"address" => "127.0.0.2", "name" => "frost", "id" => 2}).unwrap();
        let result : Vec<(u8,String, String, String)> = db.exec_and_return(String::from("SELECT * FROM lease"), mysql::Params::Empty).unwrap();
        result
    }).await.unwrap();
    println!("{:?}", bob);
    let sto = RuntimeStorage::new(db, String::from("lease"));
    let sto = Arc::new(sto);
    let synchronizer = sto.clone();
    tokio::spawn(async move {
        loop{
            synchronizer.sync(Duration::from_secs(10));
            sleep(Duration::from_secs(1)).await
        }
        
    });
    let sto = sto.clone();
    let lease = LeaseV4{
        ip_address : Ipv4Addr::BROADCAST,
        expiration : Utc::now(),
        hardware_address : HardwareAddress::new([0u8; 16]),
        mysql_table : String::from("lease"),
        id : 40
    };
    println!("{}", lease.hardware_address.to_string());
    sto.insert(lease).unwrap();
    sleep(Duration::from_secs(100)).await;

}
