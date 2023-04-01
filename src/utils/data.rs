use std::{sync::{Arc, Mutex}, collections::{HashMap, HashSet}};
use itertools::Itertools;
use mysql::{self, Pool, params, prelude::{Queryable, FromValue, FromRow}, Params, Opts};

pub trait Data {
    fn value(&self) -> params::Params;
    fn insert_statement(&self) -> String;
    fn id(&self) -> u64;
    fn location(&self) -> String;
}
pub struct DbManager{
    pub db_name : String,
    pub user : String,
    pub password : String,
    pub pool : Arc<Pool>,
}

pub struct RuntimeStorage<V : Data + FromRow>{
    runtime : Arc<Mutex<HashMap<u64,V>>>,
    dbmanager : Arc<Mutex<DbManager>>,
    table : String,
    filters : Vec<fn(&u64, &mut V) -> bool >

}

impl DbManager {
    pub fn exec_and_return<T : FromRow>(&self, stmt : String, params : Params) -> Result<Vec<T>, mysql::Error>{
        //Exec statement with given params and return result
        let pool = self.pool.clone();
        match pool.get_conn(){
            Err(e) => return Err(e),
            Ok(mut conn) => conn.exec(stmt, params)
            }
    }

    pub fn query<T : FromValue>(&self, query : String) -> Result<Vec<T>, mysql::Error> {
        //Query database
        let pool = self.pool.clone();
        match pool.get_conn(){
            Err(e) => return Err(e),
            Ok(mut conn) => conn.query(query)
        }
    }

    pub fn exec_and_drop(&self, stmt : String, params : Params) -> Result<(), mysql::Error>{
        //Exec statement with given params and drop result (usefull for dropping data for instance)
        let pool = self.pool.clone();
        match pool.get_conn(){
            Err(e) => return Err(e),
            Ok(mut conn) => conn.exec_drop(stmt, params)
            }
        }

    pub fn insert<T : Data>(&self, data : &T) -> Result<(), mysql::Error>{
        //Insert data in db
        self.exec_and_drop(data.insert_statement(), data.value())
    }

    pub fn drop<T : Data>(&self, data : &T) -> Result<(), mysql::Error>{
        //Drop data from db
        self.exec_and_drop(String::from("DELETE FROM :table WHERE id= :id"), params! {"table" => data.location(), "id" => data.id()})
    }

    pub fn new(db_name : String, user : String, password : String, host : String) -> Self{
        let url = format!("mysql://{}:{}@{}/{}", user, password, host, db_name);
        let opts = Opts::from_url(&url).unwrap();
        let pool = Pool::new(opts).unwrap();
        Self{db_name, user, password, pool : Arc::new(pool)}
    }
}



impl<'a, V : Data + FromRow + 'a> RuntimeStorage<V> where &'a V : Data{

    pub fn load(&self, database : Mutex<DbManager>){
        //Load data from database
        let db = database.lock().unwrap();
        let rows:Vec<V> = db.exec_and_return(String::from("SELECT * FROM :table "), params! {"table" => &self.table}).unwrap();
        for element in rows.into_iter(){
            let mut db = self.runtime.lock().unwrap();
            if !db.contains_key(&element.id()){
                db.insert(element.id(), element);
            }
        }
    }

    fn database_sync(&'a self) -> Result<(), mysql::Error>{
        //Sync database with runtime
        let db = self.dbmanager.lock().unwrap();
        //Compute ids stored on disk
        let disk_ids:Vec<u64> = db.exec_and_return(format!("SELECT id FROM {} ", &self.table), Params::Empty).unwrap();
        let disk_ids : HashSet<u64> = disk_ids.iter().cloned().collect();
        //Compute ids in runtime
        let runtime = self.runtime.lock().unwrap();
        let runtime_ids : HashSet<u64> = runtime.keys().cloned().collect();
        //Set differences
        let deprecated_ids = &disk_ids - &runtime_ids;
        let new_ids = &runtime_ids - &disk_ids;

        //Add new ids to disk
        for id in new_ids {
            let value = runtime.get(&id).unwrap();
            db.insert(value).unwrap();
        };

        let ids = deprecated_ids.iter().join(",");
        //Remove old ids from disk
        if ids.len() > 0 {
            return db.exec_and_drop(format!("DELETE FROM {} WHERE id IN ( {} )",&self.table, ids),Params::Empty)
        }else {
            return Ok(());
        }
        
    }

    pub fn store(&self, data : V)-> Result<(), &str>{
        //Store data
        let mut runtime = self.runtime.lock().unwrap();
        if !runtime.contains_key(&data.id()){
            runtime.insert(data.id(), data);
            Ok(())
        }else {
            Err("Id already in use")
        }
    }

    pub fn new(db : Arc<Mutex<DbManager>>, table : String) -> Self{
        Self { runtime: Arc::new(Mutex::new(HashMap::new())), dbmanager: db.clone(), table, filters : vec![]}
    }

    pub fn sync(&'a self){
        //Run every sync task
        self.database_sync().unwrap();
        //Filter data
        for &filter in &self.filters {
            self.purge(filter);
        }
    }

    fn purge(&self, filter : fn(&u64, &mut V) -> bool){
        //Delete object based on filter
        let mut hashmap = self.runtime.lock().unwrap();
        hashmap.retain(filter);
    }

    pub fn add_filter(&mut self, filter : fn(&u64, &mut V) -> bool){
        //Add filter to filters
        self.filters.push(filter);
    }

}


