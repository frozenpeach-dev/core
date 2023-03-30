pub mod core;
pub mod hooks;
pub mod utils;
use std::sync::{Mutex, Arc};


use mysql::params;
use tokio;

use utils::data::DbManager;

#[tokio::main]
async fn main() {
    let db = Arc::new(Mutex::new(DbManager::new(String::from("dhcp"), String::from("frozenpeach"), String::from("poney"), String::from("127.0.0.1:3333"))));
    let database = db.clone();
    let bob =tokio::spawn(async move {
        let db = database.lock().unwrap();
        //let result : Vec<String> = db.exec_and_return(String::from("INSERT INTO lease VALUE (:address, :name, :id)"), params! {"address" => "127.0.0.2", "name" => "frost", "id" => 2}).unwrap();
        let result : Vec<(String, String, u8)> = db.exec_and_return(String::from("SELECT * FROM lease"), mysql::Params::Empty).unwrap();
        result
    }).await.unwrap();
    println!("{:?}", bob);
}
