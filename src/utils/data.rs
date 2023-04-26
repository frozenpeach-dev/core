//! This module provides tools to store your data with a mysql synchronization
use std::{sync::{Arc, Mutex}, collections::{HashMap, HashSet, hash_map::Entry}};
use itertools::Itertools;
use mysql::{self, Pool, params, prelude::{Queryable, FromValue, FromRow}, Params, Opts};
use log;
use rand;

///Trait implementing methods for data that will be stored in RuntimeStorage.
pub trait Storable {
    fn value(&self) -> params::Params;
    fn insert_statement(&self, place : String) -> String;
    fn id(&self) -> u16;
    fn set_uid(&mut self, uid : u16);
}

///DbManager aims to manage MySql connections and interactions.
pub struct DbManager{
    pub db_name : String,
    pub user : String,
    pub password : String,
    pub pool : Arc<Pool>,
}

///RuntimeStorage manage storage. It is the interface between user and runtime/backend storage.
pub struct RuntimeStorage<V : Storable + Clone>{
    pools : Arc<Mutex<HashMap<String, Arc<Mutex<DataPool<V>>>>>>,
    dbmanager : Arc<Mutex<DbManager>>,
    index : Arc<Mutex<HashMap<u16, String>>>
}

///`DataPool` is a high-level storage manager tha allows you to quickly access and store data, while ensuring your data are protected from code interruption with live MySql Database synchronization.
pub struct DataPool<V : Storable>{
    name : String,
    filters : Vec<fn(&u16, &V) -> bool>,
    runtime : Arc<Mutex<HashMap<u16,V>>>,
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
    pub fn insert<V : Storable>(&self, data : &V, place: String) -> Result<(), mysql::Error>{
        //Insert data in db
        self.exec_and_drop(data.insert_statement(place), data.value())
    }

    ///Drop data having given id. A table must be given.
    pub fn drop(&self, table : String, ids : Vec<u16>) -> Result<(), mysql::Error>{
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



impl<V : Storable + Clone + FromRow> RuntimeStorage<V>{

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
     ///Get data from disk storage given its UID
    pub fn get_from_disk(&self, uid: u16) -> Result<V, String>{
        let index = self.index.clone();
        let index = index.lock().unwrap();
        let pool = index.get(&uid).ok_or_else(|| String::from("UID doesn't exist in any pool"))?;
        let db = self.dbmanager.clone();
        let db = db.lock().unwrap();
        let data : Vec<V> = db.exec_and_return(format!("SELECT * FROM {} WHERE id = {}", pool, uid), Params::Empty).unwrap();
        match data.len(){
            0 => Err(String::from("No data with given uid")),
            _ => Ok(data[0].clone())
        }
    }

    /// Delete data given its id
    pub fn delete(&mut self, id: u16, pool_name : String) {
        let pools = self.pools.clone();
        let pools = pools.lock().unwrap();
        let pool = pools.get(&pool_name).unwrap().clone();
        let pool = pool.lock().unwrap();
        pool.delete(&id)
    }

    pub fn get(&self, uid : u16)-> Result<V, String>{
        let index = self.index.clone();
        let index = index.lock().unwrap();
        let pool = index.get(&uid).unwrap();
        let pools = self.pools.clone();
        let pools = pools.lock().unwrap();
        let pool = pools.get(pool).unwrap().clone();
        let pool = pool.lock().unwrap();
        let data = pool.get(uid).ok_or_else(|| String::from("No current data for given id..."));
        data

    }

    ///Synchronizes given pool with database : inserts missing data in database and remove old data 
    fn pool_sync(&self, pool : &Arc<Mutex<DataPool<V>>>) -> Result<(), mysql::Error>{
        //Sync database with runtime
        let db = self.dbmanager.lock().unwrap();
        let pool = pool.clone();
        let pool = pool.lock().unwrap();
        //Compute ids stored on disk
        let disk_ids:Vec<u16> = db.exec_and_return(format!("SELECT id FROM {} ", pool.name), Params::Empty)?;
        let disk_ids : HashSet<u16> = disk_ids.iter().cloned().collect();
        //Compute ids in runtime
        let runtime = pool.runtime.lock().unwrap();
        let runtime_ids : HashSet<u16> = runtime.keys().cloned().collect();
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

    ///Generate uid
    fn get_unused_id(&self) -> u16{
        let index = self.index.clone();
        let index = index.lock().unwrap();
        let uid = {
            let mut rd : u16 = rand::random();
            while (&index).contains_key(&rd){
                 rd = rand::random();
            }
            rd
        };
        uid
    }

    /// Store data in the pool given the pool name and return an uid representing the data. The uid is unique among all pools.
    /// Example
    /// ```rust
    /// runtime.store(data, String::from("pool_name"));
    /// ```
    pub fn store(&mut self, mut data : V, pool_name : String)-> Result<u16, String>{
        //Store data
        let uid = self.get_unused_id();
        let pool = self.pools.clone().lock().unwrap().get(&pool_name).unwrap().clone();
        let pool = pool.lock().unwrap();
        data.set_uid(uid);
        self.index.clone().lock().unwrap().insert(uid, pool.name());
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
    ///     loop {
    ///         time::sleep(duration).await;
    ///         synchronizer.lock().unwrap().sync();
    ///     }
    /// }).await;
    /// ```
    pub fn sync(&mut self){
        let mut removed_overall:Vec<u16> = vec![];
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

impl<V : Storable + FromRow + Clone> DataPool<V>{
    ///Iter over filters and drop data that return false when passed as argument to condition functions.
    pub fn purge(&self) -> Vec<u16>{
        let mut overall_removed: Vec<u16> = vec![];
        log::info!("Purging pool {}", self.name);
        for filter in &self.filters {
            let mut removed: Vec<u16> = vec![];
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
    pub fn add_filter(&mut self, filter : fn(&u16, &V) -> bool){
        //Add filter to filters
        self.filters.push(filter);
    }

    ///Inserts data in a pool, this function is private, meaning that to store data in a pool, you would use :
    /// ```ignore
    /// let data = Data::new();
    /// dataPool.store(data, pool_name);
    /// ```
    fn insert(&self, data : V) -> Result<u16, String>{
        let mut runtime = self.runtime.lock().unwrap();
        if let Entry::Vacant(e) = runtime.entry(data.id()) {
            let id = data.id();
            e.insert(data);
            Ok(id)
        } else {
            Err(String::from("Id already in use"))
        }
    }

    fn get(&self, uid : u16) -> Option<V>{
        let runtime = self.runtime.lock().unwrap();
        runtime.get(&uid).cloned()
    }

    ///Drops data given its id.
    fn drop(&self, id : &u16){
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

#[cfg(test)]
mod test {
    use derive_data::Storable;
    use std::time::{Duration, Instant};
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Lease {
        name :String,
        address : String,
        uid : u16
    }

    impl Storable for Lease{
        fn id(&self) -> u16 {
            self.uid.clone()
        }
        fn insert_statement(&self, place : String) -> String {
            format!("INSERT INTO {} VALUE ( :type, :id, :name, :address)", place)
        }
        fn set_uid(&mut self, uid : u16) {
            self.uid = uid;
        }
        fn value(&self) -> params::Params {
            let name = self.name.clone();
            let uid = self.uid;
            let address = self.address.clone();
            params! {"type" => "lease", "id" => uid, "name" => name, "address" => address}
        }
    }

    impl FromRow for Lease{
        fn from_row(row: mysql::Row) -> Self
            where
                Self: Sized, {
            let id : u16= row.get(1).unwrap();
            let name:String = row.get(2).unwrap();
            let address = row.get(3).unwrap();
            Self { name, address, uid: id }
        }

        fn from_row_opt(row: mysql::Row) -> Result<Self, mysql::FromRowError>
            where
                Self: Sized {
                    let id : u16 = row.get(1).unwrap();
                    let name:String = row.get(2).unwrap();
                    let address :String= row.get(3).unwrap();
                    Ok(Self { name, address, uid: id }) 
            
        }
    }

    #[derive(Clone, Storable, PartialEq, Eq)]
    pub enum Data {
        Lease(Lease),
        Null,
    }

    impl FromRow for Data {
        fn from_row(row: mysql::Row) -> Self
            where
                Self: Sized, {
            let data : String = row.get(0).unwrap();
            match data.as_str() {
                "lease" => return Data::Lease(Lease::from_row(row)),
                _ => Data::Null
            }
        }

        fn from_row_opt(row: mysql::Row) -> Result<Self, mysql::FromRowError>
            where
                Self: Sized {
            let data : String = row.get(0).unwrap();
            match data.as_str() {
                "lease" => {
                    let opt = Lease::from_row_opt(row);
                    match opt {
                        Ok(lease) => return Ok(Data::Lease(lease)),
                        Err(e) => return Err(e)
                    }
                },
                _ => Ok(Data::Null)
            }
        }
    }


    async fn insert_retrieve_benchmark(bench : Arc<Mutex<RuntimeStorage<Data>>>){
        
        let lease = Lease{
            name : String::from("test"),
            address : String::from("127.0.0.1"),
            uid : 0
        };
        let lease = Data::Lease(lease);

        //Insert nb lease
        let nb = 1000u16;
        let manager = bench.clone();
        let ids = tokio::spawn(async move {
            println!("Starting {} insertions...", nb);
            let start = Instant::now();
            let mut ids = vec![];
            let mut manager = manager.lock().unwrap();
            for _i in 0..nb{
                let id  = manager.store(lease.clone(), String::from("lease")).unwrap();
                ids.push(id);
            }
            println!("Inserted {} data in {:.2?}",ids.len(),  start.elapsed());
            ids
        }).await.unwrap();

        let ids_disk = ids.clone();

        //Retrieve from runtime
        let getter = bench.clone();
        tokio::spawn(async move {
            let start = Instant::now();
            let mut datas = vec![];
            for id in ids{
                datas.push(getter.lock().unwrap().get(id).unwrap());
            }
            println!("Retrieved from runtime in {:.2?}", start.elapsed());
            datas
            
        }).await.unwrap();
        
        //Retrieve from disk
        tokio::time::sleep(Duration::from_millis(10)).await;
        let disk_getter = bench.clone();
        tokio::spawn(async move {
            let start = Instant::now();
            println!("Retrieving {} datas from disk...", ids_disk.len());
            let mut datas = vec![];
            for id in ids_disk {
                datas.push(disk_getter.lock().unwrap().get_from_disk(id).unwrap());
            }
            println!("Retrieved from disk in {:.2?}", start.elapsed());
            datas
        }).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_data_storage(){
        println!("");
        //Create RuntimeStorage
        let db = DbManager::new(String::from("dhcp"), String::from("frozenpeach"), String::from("poney"), String::from("127.0.0.1:3333"));
        let storage: RuntimeStorage<Data> = RuntimeStorage::new(Arc::new(Mutex::new(db)));
        let storage = Arc::new(Mutex::new(storage));
        let sync = storage.clone();
        let manager = storage.clone();
        
        let lease = Lease{
            name : String::from("test"),
            address : String::from("127.0.0.1"),
            uid : 0
        };
        let lease = Data::Lease(lease);
        
        //Create pool and insert data
        let id = tokio::spawn(async move {
            let lease_pool = DataPool::new(String::from("lease"), String::from("(id BIGINT, name VARCHAR(255), address VARCHAR(255))"));
            let mut manager = manager.lock().unwrap();
            manager.add_pool(lease_pool);
            let id  = manager.store(lease, String::from("lease")).unwrap();
            return id
        }).await.unwrap();
        
        //Start sync
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(10)).await;
                sync.lock().unwrap().sync();
            }
        });
        
        //Get from disk and from runtime
        let getter = storage.clone();
        tokio::time::sleep(Duration::from_millis(200)).await;
        let (data1, data2) = tokio::spawn(async move {
            let data1 = getter.lock().unwrap().get_from_disk(id).unwrap();
            let data2 = getter.lock().unwrap().get(id).unwrap();
            (data1, data2)
            
        }).await.unwrap();

        //Ensure Runtime and Disk have same info
        assert!(data1 == data2);

        //Run benchmark
        let bench = storage.clone();
        insert_retrieve_benchmark(bench).await;
    }


}

