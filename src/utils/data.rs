//! This module provides tools to store your data with a mysql synchronization
use std::{sync::{Arc, Mutex}, collections::{HashMap, HashSet, hash_map::Entry}};
use itertools::Itertools;
use mysql::{self, Pool, params, prelude::{Queryable, FromValue, FromRow}, Params, Opts};
use log;

///Trait implementing methods for data that will be stored in RuntimeStorage.
pub trait Data {
    fn value(&self) -> params::Params;
    fn insert_statement(&self, place : String) -> String;
    fn id(&self) -> u64;
}

///DbManager aims to manage MySql connections and interactions.
pub struct DbManager{
    pub db_name : String,
    pub user : String,
    pub password : String,
    pub pool : Arc<Pool>,
}

///RuntimeStorage manage storage. It is the interface between user and runtime/backend storage.
pub struct RuntimeStorage<V : Data + FromRow>{
    pools : Arc<Mutex<HashMap<String, Arc<Mutex<DataPool<V>>>>>>,
    dbmanager : Arc<Mutex<DbManager>>,
    index : Arc<Mutex<HashMap<u64, String>>>
}

///`DataPool` is a high-level storage manager tha allows you to quickly access and store data, while ensuring your data are protected from code interruption with live MySql Database synchronization.
pub struct DataPool<V : Data + FromRow> {
    name : String,
    filters : Vec<fn(&u64, &V) -> bool>,
    runtime : Arc<Mutex<HashMap<u64,V>>>,
    schema : String
}

impl DbManager {
    ///Exec statement with given params and return the result
    pub fn exec_and_return<T : FromRow>(&self, stmt : String, params : Params) -> Result<Vec<T>, mysql::Error>{
        //Exec statement with given params and return result
        let pool = self.pool.clone();
        match pool.get_conn(){
            Err(e) => return Err(e),
            Ok(mut conn) => conn.exec(stmt, params)
        }
    }

    ///Exec guven query.
    pub fn query<T : FromValue>(&self, query : String) -> Result<Vec<T>, mysql::Error> {
        //Query database
        let pool = self.pool.clone();
        pool.get_conn()?.query(query)
    }

    ///Exec statement with given params and drop the result (usefull for drop statement for example)
    fn exec_and_drop(&self, stmt : String, params : Params) -> Result<(), mysql::Error>{
        //Exec statement with given params and drop result (useful for dropping data for instance)
        let pool = self.pool.clone();
        pool.get_conn()?.exec_drop(stmt, params)
    }

    ///Insert data in a given table
    pub fn insert<T : Data>(&self, data : &T, place: String) -> Result<(), mysql::Error>{
        //Insert data in db
        self.exec_and_drop(data.insert_statement(place), data.value())
    }

    ///Drop data having given id. A table must be given.
    pub fn drop<T : Data>(&self, table : String, ids : Vec<u64>) -> Result<(), mysql::Error>{
        //Drop data from db
        self.exec_and_drop(String::from("DELETE FROM :table WHERE id = :id"), params! {"table" => table, "id" => ids.iter().join(",")})
    }

    pub fn new(db_name : String, user : String, password : String, host : String) -> Self{
        let url = format!("mysql://{}:{}@{}/{}", user, password, host, db_name);
        let opts = Opts::from_url(&url).unwrap();
        let pool = Pool::new(opts).unwrap();
        Self { db_name, user, password, pool : Arc::new(pool) }
    }
}



impl<'a, V : Data + FromRow + 'a> RuntimeStorage<V> where &'a V : Data{

    ///Load data from static mysql database.
    pub fn load(&mut self, database : Mutex<DbManager>){
        //Load data from database
        let db = database.lock().unwrap();
        let tables:Vec<String> = db.exec_and_return(String::from("SHOW TABLES"), Params::Empty).unwrap();
        for table in tables {
            let pool = DataPool::empty(table.clone());
            self.add_pool(pool);
            let rows:Vec<V> = db.exec_and_return(String::from("SELECT * FROM :table "), params! {"table" => table.clone()}).unwrap();
            for data in rows {
                let id = data.id();
                if !self.index.clone().lock().unwrap().contains_key(&data.id()){
                    self.store(data, table.clone()).unwrap();
                    log::info!("Loaded data {}", id);
                } else {
                    log::info!("Tried to load already existing data : {}", id);
                }
            }
        }
    }

    ///Synchronizes given pool with database : inserts missing data in database and remove old data 
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
            db.insert(value, String::from(self.index.clone().lock().unwrap().get(&id).unwrap())).unwrap();
        };

        let ids = deprecated_ids.iter().join(",");
        //Remove old ids from disk
        if !ids.is_empty() {
            db.exec_and_drop(format!("DELETE FROM {} WHERE id IN ( {} )",pool.name, ids),Params::Empty)
        } else {
            Ok(())
        }
        
    }

    /// Store data in the pool given the pool name.
    /// Example
    /// ```rust
    /// runtime.store(data, String::from("pool_name"));
    /// ```
    pub fn store(&mut self, data : V, pool_name : String)-> Result<(), String>{
        //Store data
        let pool = self.pools.clone().lock().unwrap().get(&pool_name).unwrap().clone();
        let pool = pool.lock().unwrap();
        self.index.clone().lock().unwrap().insert(data.id(), pool.name());
        pool.insert(data)
        
    }

    pub fn new(db : Arc<Mutex<DbManager>>) -> Self{
        Self { dbmanager: db, pools : Arc::new(Mutex::new(HashMap::new())), index : Arc::new(Mutex::new(HashMap::new()))}
    }

    ///Run every task for synchronization.
    /// To synchronize your RuntimeStorage, you will need to use something like :
    /// ```rust
    /// let runtime = RuntimeStorage::new();
    /// let runtime = Arc::new(Mutex::new(runtime));
    /// let synchronizer = runtime.clone();
    /// tokio::spawn(async move {
    ///     looop {
    ///         time::sleep(duration).await;
    ///         synchronizer.lock().unwrap().sync();
    ///     }
    /// }).await;
    /// ```
    pub fn sync(&'a mut self){
        let mut removed_overall:Vec<u64> = vec![];
        for pool in self.pools.clone().lock().unwrap().values() {
            //Run every sync task
            self.pool_sync(pool).unwrap();
            //Filter data
            let mut removed = pool.clone().lock().unwrap().purge();
            removed_overall.append(&mut removed);
            
        }
        for k in removed_overall {
            self.index.clone().lock().unwrap().remove(&k);
        };
    }

    ///Add a pool `DataPool` to storage.
    /// # Example
    /// ```rust
    /// let pool = DataPool::new();
    /// runtime.add_pool(pool);
    /// ```
    pub fn add_pool(&self, pool : DataPool<V>) {
        let mut pools = self.pools.lock().unwrap();
        let name = pool.name();
        let schema = pool.schema();
        pools.insert(name.clone(), Arc::new(Mutex::new(pool)));
        self.dbmanager.lock().unwrap().exec_and_drop(format!("CREATE TABLE IF NOT EXISTS {} {}", name, schema), Params::Empty).unwrap();
    }

}

impl<V : Data + FromRow> DataPool<V>{
    ///Iter over filters and drop data that return false when passed as argument to condition functions.
    pub fn purge(&self) -> Vec<u64>{
        let mut overall_removed: Vec<u64> = vec![];
        log::info!("Purging pool {}", self.name);
        for filter in &self.filters {
            let mut removed: Vec<u64> = vec![];
            let mut data = self.runtime.lock().unwrap();
            for (k, v) in data.iter(){
                if filter(&k,&v){
                    removed.push(*k);
                }
            }
            for k in removed.iter(){
                data.remove(k);
            }
            overall_removed.append(& mut removed);
        }
        overall_removed
    }      

    ///Add filter to filter list.
    pub fn add_filter(&mut self, filter : fn(&u64, &V) -> bool){
        //Add filter to filters
        self.filters.push(filter);
    }

    ///Inserts data in a pool, this function is private, meaning that to store data in a pool, you would use :
    /// ```ignore
    /// let data = Data::new();
    /// dataPool.store(data, pool_name);
    /// ```
    fn insert(&self, data : V) -> Result<(), String>{
        let mut runtime = self.runtime.lock().unwrap();
        if let Entry::Vacant(e) = runtime.entry(data.id()) {
            e.insert(data);
            Ok(())
        } else {
            Err(String::from("Id already in use"))
        }
    }

    ///Drops data given its id.
    fn drop(&self, id : &u64){
        self.runtime.lock().unwrap().remove(id);
    }

    ///Create an empty pool with a given name.
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

    ///Getter
    pub fn name(&self)-> String{
        self.name.clone()
    }

    ///Getter
    pub fn schema(&self) -> String{
        self.schema.clone()
    }
}
