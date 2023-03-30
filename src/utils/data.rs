use std::{sync::Arc};
use tokio::{self, task::JoinError};
use mysql::{self, Pool, params, prelude::Queryable, Params, Opts};


pub struct DbManager {
    pub db_name : String,
    pub user : String,
    pub password : String,
    pub pool : Arc<Pool>

}

pub trait Data {
    fn value(&self) -> params::Params;
    fn insert_statement(&self) -> String;
}

impl DbManager {
    pub async fn exec(&self, stmt : String, params : Params) -> Result<Result<(), mysql::Error>, JoinError>{
        let pool = self.pool.clone();
        tokio::spawn(async move {
            match pool.get_conn(){
                Err(e) => return Err(e),
                Ok(mut conn) => conn.exec_drop(stmt, params)
                }
        }).await
    }

    pub async fn insert<T : Data>(&self, data : T) -> Result<Result<(), mysql::Error>, JoinError>{
        self.exec(data.insert_statement(), data.value()).await
    }

    pub fn new(db_name : String, user : String, password : String, host : String) -> Self{
        let url = format!("mysql://{}:{}@{}/{}", user, password, host, db_name);
        let opts = Opts::from_url(&url).unwrap();
        let pool = Pool::new(opts).unwrap();
        Self{db_name, user, password, pool : Arc::new(pool)}
    }
}

unsafe impl Send for DbManager{}
unsafe impl Sync for DbManager{}