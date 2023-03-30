use std::{sync::{Arc, Mutex}, collections::{HashMap, HashSet}};
use chrono::Duration;
use itertools::Itertools;
use mysql::{self, Pool, params, prelude::{Queryable, FromValue, FromRow}, Params, Opts};
use tokio::{runtime, time::sleep};

pub struct DbManager {
    pub db_name : String,
    pub user : String,
    pub password : String,
    pub pool : Arc<Pool>

}

pub trait Data {
    fn value(&self) -> params::Params;
    fn insert_statement(&self) -> String;
    fn id(&self) -> u64;
}

impl DbManager {
    pub fn exec_and_return<T : FromRow>(&self, stmt : String, params : Params) -> Result<Vec<T>, mysql::Error>{
    let pool = self.pool.clone();
    match pool.get_conn(){
        Err(e) => return Err(e),
        Ok(mut conn) => conn.exec(stmt, params)
        }
    }

    pub fn query<T : FromValue>(&self, query : String) -> Result<Vec<T>, mysql::Error> {
        let pool = self.pool.clone();
        match pool.get_conn(){
            Err(e) => return Err(e),
            Ok(mut conn) => conn.query(query)
        }
    }

    pub fn exec_and_drop(&self, stmt : String, params : Params) -> Result<(), mysql::Error>{
        let pool = self.pool.clone();
        match pool.get_conn(){
            Err(e) => return Err(e),
            Ok(mut conn) => conn.exec_drop(stmt, params)
            }
        }

    pub fn insert<T : Data>(&self, data : T) -> Result<(), mysql::Error>{
        self.exec_and_drop(data.insert_statement(), data.value())
    }

    pub fn new(db_name : String, user : String, password : String, host : String) -> Self{
        let url = format!("mysql://{}:{}@{}/{}", user, password, host, db_name);
        let opts = Opts::from_url(&url).unwrap();
        let pool = Pool::new(opts).unwrap();
        Self{db_name, user, password, pool : Arc::new(pool)}
    }
}

pub struct RuntimeStorage<V : Data + FromRow + Copy >{
    storage : Arc<Mutex<HashMap<u64,V>>>,
    dbmanager : Arc<Mutex<DbManager>>,
    table : String
}

impl<'a, V : Data + FromRow + Copy + 'a> RuntimeStorage<V> where &'a V : Data{

    pub fn load(&self, database : Mutex<DbManager>, table : String){
        let db = database.lock().unwrap();
        let rows:Vec<V> = db.exec_and_return(format!("SELECT * FROM {}", table), Params::Empty).unwrap();
        for element in rows.into_iter(){
            let mut db = self.storage.lock().unwrap();
            if !db.contains_key(&element.id()){
                db.insert(element.id(), element);
            }
        }
    }

    pub fn database_sync(&'a self) -> Result<(), mysql::Error>{
        let db = self.dbmanager.lock().unwrap();
        let disk_ids:Vec<u64> = db.exec_and_return(String::from("SELECT :id FROM :table"), params! {"table" => &table, "id" => "id"}).unwrap();
        let disk_ids : HashSet<u64> = disk_ids.iter().cloned().collect();
        let runtime = self.storage.lock().unwrap();
        let runtime_ids : HashSet<u64> = runtime.keys().cloned().collect();
        let deprecated_ids = &disk_ids - &runtime_ids;
        let new_ids = &runtime_ids - &disk_ids;
        for id in new_ids {
            let value = *runtime.get(&id).unwrap();
            db.insert(value).unwrap();
        };
        let ids = deprecated_ids.iter().join(",");
        db.exec_and_drop(String::from("DELETE FROM :table WHERE id IN ( :id )"),params!{"table" => &table, "id" => ids})
    }

    pub fn insert(&self, data : V)-> Result<(), &str>{
        let mut runtime = self.storage.lock().unwrap();
        if !runtime.contains_key(&data.id()){
            runtime.insert(data.id(), data);
            Ok(())
        }else {
            Err("Id already in use")
        }
    }



    pub async fn start_sync(&self, delay : std::time::Duration){
        todo!();
    }



}


