use std::{sync::{Arc, Mutex}, collections::{HashMap, HashSet, hash_map::Entry}};
use itertools::Itertools;
use mysql::{self, Pool, params, prelude::{Queryable, FromValue, FromRow}, Params, Opts};

pub trait Data {
    fn value(&self) -> params::Params;
    fn insert_statement(&self, place : String) -> String;
    fn id(&self) -> u64;
}
pub struct DbManager{
    pub db_name : String,
    pub user : String,
    pub password : String,
    pub pool : Arc<Pool>,
}

pub struct RuntimeStorage<V : Data + FromRow>{
    pools : Arc<Mutex<HashMap<String, Arc<Mutex<DataPool<V>>>>>>,
    dbmanager : Arc<Mutex<DbManager>>,
    index : HashMap<u64, String>
}

pub struct DataPool<V : Data + FromRow> {
    name : String,
    filters : Vec<fn(&u64, &mut V) -> bool>,
    runtime : Arc<Mutex<HashMap<u64,V>>>,
    schema : String
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

    pub fn insert<T : Data>(&self, data : &T, place: String) -> Result<(), mysql::Error>{
        //Insert data in db
        self.exec_and_drop(data.insert_statement(place), data.value())
    }

    pub fn drop<T : Data>(&self, table : String, ids : Vec<u64>) -> Result<(), mysql::Error>{
        //Drop data from db
        self.exec_and_drop(String::from("DELETE FROM :table WHERE id = :id"), params! {"table" => table, "id" => ids.iter().join(",")})
    }

    pub fn new(db_name : String, user : String, password : String, host : String) -> Self{
        let url = format!("mysql://{}:{}@{}/{}", user, password, host, db_name);
        let opts = Opts::from_url(&url).unwrap();
        let pool = Pool::new(opts).unwrap();
        Self{db_name, user, password, pool : Arc::new(pool)}
    }
}



impl<'a, V : Data + FromRow + 'a> RuntimeStorage<V> where &'a V : Data{

    // pub fn load(&self, database : Mutex<DbManager>){
    //     //Load data from database
    //     let db = database.lock().unwrap();
    //     let rows:Vec<V> = db.exec_and_return(String::from("SELECT * FROM :table "), params! {"table" => &self.table}).unwrap();
    //     for element in rows.into_iter(){
    //         let mut db = self.runtime.lock().unwrap();
    //         if !db.contains_key(&element.id()){
    //             db.insert(element.id(), element);
    //         }
    //     }
    // }

    fn pool_sync(&'a self, pool : &Arc<Mutex<DataPool<V>>>) -> Result<(), mysql::Error>{
        //Sync database with runtime
        let db = self.dbmanager.lock().unwrap();
        let pool = pool.clone();
        let pool = pool.lock().unwrap();
        //Compute ids stored on disk
        let disk_ids:Vec<u64> = db.exec_and_return(format!("SELECT id FROM {} ", pool.name), Params::Empty).unwrap();
        let disk_ids : HashSet<u64> = disk_ids.iter().cloned().collect();
        //Compute ids in runtime
        let runtime = pool.runtime.lock().unwrap();
        let runtime_ids : HashSet<u64> = runtime.keys().cloned().collect();
        //Set differences
        let deprecated_ids = &disk_ids - &runtime_ids;
        let new_ids = &runtime_ids - &disk_ids;

        //Add new ids to disk
        for id in new_ids {
            let value = runtime.get(&id).unwrap();
            db.insert(value, String::from(self.index.get(&id).unwrap())).unwrap();
        };

        let ids = deprecated_ids.iter().join(",");
        //Remove old ids from disk
        if ids.len() > 0 {
            return db.exec_and_drop(format!("DELETE FROM {} WHERE id IN ( {} )",pool.name, ids),Params::Empty)
        }else {
            return Ok(());
        }
        
    }

    pub fn store(&mut self, data : V, pool_name : String)-> Result<(), String>{
        //Store data
        let pool = self.pools.clone().lock().unwrap().get(&pool_name).unwrap().clone();
        let pool = pool.lock().unwrap();
        self.index.insert(data.id(), pool.name());
        pool.insert(data)
        
    }

    pub fn new(db : Arc<Mutex<DbManager>>) -> Self{
        Self { dbmanager: db.clone(), pools : Arc::new(Mutex::new(HashMap::new())), index : HashMap::new()}
    }

    pub fn sync(&'a self){
        for pool in self.pools.clone().lock().unwrap().values() {
            //Run every sync task
            self.pool_sync(pool).unwrap();
            //Filter data
            pool.clone().lock().unwrap().purge();
        }
    }
    
    pub fn add_pool(&self, pool : DataPool<V>) {
        let mut pools = self.pools.lock().unwrap();
        let name = pool.name();
        let schema = pool.schema();
        pools.insert(name.clone(), Arc::new(Mutex::new(pool)));
        self.dbmanager.lock().unwrap().exec_and_drop(format!("CREATE TABLE IF NOT EXISTS {} {}", name, schema), Params::Empty).unwrap();
    }

}

impl<V : Data + FromRow> DataPool<V>{
    pub fn purge(&self){
        println!("Purging pool {}", self.name);
        for filter in &self.filters {
            let mut data = self.runtime.lock().unwrap();
            data.retain(filter);
        }
    }

    pub fn add_filter(&mut self, filter : fn(&u64, &mut V) -> bool){
        //Add filter to filters
        self.filters.push(filter);
    }

    pub fn insert(&self, data : V) -> Result<(), String>{
        let mut runtime = self.runtime.lock().unwrap();
        if !runtime.contains_key(&data.id()){
            runtime.insert(data.id(), data);
            Ok(())
        }else {
            Err(String::from("Id already in use"))
        }
        if let Entry::Vacant(e) = runtime.entry(data.id()) {
            e.insert(data);
            Ok(())
        } else {
            Err(String::from("Id already in use"))
        }
        
    }

    pub fn drop(&self, id : &u64){
        self.runtime.lock().unwrap().remove(id);
    }

    pub fn empty(name : String) -> Self {
        Self {
            name,
            filters : vec![],
            runtime : Arc::new(Mutex::new(HashMap::new())),
            schema : String::from("(id INT)")
        }
    }

    pub fn new(name: String, schema : String) -> Self{
        Self {
            name,
            filters : vec![],
            runtime : Arc::new(Mutex::new(HashMap::new())),
            schema
        }
    }

    pub fn name(&self)-> String{
        self.name.clone()
    }

    pub fn schema(&self) -> String{
        self.schema.clone()
    }
}
