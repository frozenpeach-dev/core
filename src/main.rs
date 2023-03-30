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
    tokio::spawn(async move {
        let db = database.lock().unwrap();
        db.exec(String::from("INSERT INTO lease VALUE (:address, :name, :id)"), params! {"address" => "127.0.0.2", "name" => "frost", "id" => 2}).unwrap();
    }).await.unwrap();
}
