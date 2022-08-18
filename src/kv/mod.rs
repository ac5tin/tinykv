use std::{collections::HashMap, sync::RwLock};

use actix::prelude::*;
use anyhow::anyhow;
use sea_orm::DatabaseConnection;

use crate::db::DB;

#[derive(Message, Clone)]
#[rtype(result = "Result<(), anyhow::Error>")]
pub struct Dataset {
    pub key: String,
    pub data: Vec<u8>,
}

#[derive(Message)]
#[rtype(result = "Result<Vec<u8>, anyhow::Error>")]
pub struct Key(pub String);

pub struct KvStore {
    data: HashMap<String, RwLock<Vec<u8>>>,
    db: Addr<DB>,
}

impl KvStore {
    pub fn new(conn: DatabaseConnection) -> KvStore {
        KvStore {
            data: HashMap::new(),
            db: DB::new(conn).start(),
        }
    }
}

impl Actor for KvStore {
    type Context = Context<Self>;
}

impl Handler<Dataset> for KvStore {
    type Result = Result<(), anyhow::Error>;

    fn handle(&mut self, msg: Dataset, _: &mut Self::Context) -> Self::Result {
        match self.data.get(&msg.key) {
            Some(d) => {
                // update data behind rwlock
                // acquire lock
                if let Ok(mut wlock) = d.write() {
                    *wlock = msg.data.clone();
                    if let Err(err) = self.db.try_send(msg.to_owned()) {
                        log::error!("Failed to persist data in database, Err:{:?}", err);
                        return Err(anyhow!(err));
                    };
                    Ok(())
                } else {
                    Err(anyhow!("failed to acquire write lock"))
                }
            }
            None => {
                // no key found, create new entry
                self.data.insert(msg.key, RwLock::new(msg.data));
                Ok(())
            }
        }
    }
}

impl Handler<Key> for KvStore {
    type Result = Result<Vec<u8>, anyhow::Error>;

    fn handle(&mut self, msg: Key, _: &mut Self::Context) -> Self::Result {
        match self.data.get(&msg.0) {
            Some(d) => {
                // acquire lock
                if let Ok(rlock) = d.read() {
                    Ok((*rlock.to_owned()).to_vec())
                } else {
                    Err(anyhow!("failed to acquire read lock"))
                }
            }
            None => Err(anyhow!("key not found")),
        }
    }
}
