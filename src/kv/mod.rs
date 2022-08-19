use actix::prelude::*;
use actix_interop::{with_ctx, FutureInterop};
use anyhow::anyhow;
use lru::LruCache;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::DB;

#[derive(Message, Clone)]
#[rtype(result = "Result<(), anyhow::Error>")]
pub struct Dataset {
    pub key: String,
    pub data: Vec<u8>,
}

#[derive(Message, Clone)]
#[rtype(result = "Result<Vec<u8>, anyhow::Error>")]
pub struct Key(pub String);

pub struct KvStore {
    cache: Arc<RwLock<LruCache<String, Vec<u8>>>>,
    db: Addr<DB>,
}

impl KvStore {
    pub fn new(conn: DatabaseConnection) -> KvStore {
        KvStore {
            cache: Arc::new(RwLock::new(LruCache::new(100))),
            db: DB::new(conn).start(),
        }
    }
}

impl Actor for KvStore {
    type Context = Context<Self>;
}

impl Handler<Dataset> for KvStore {
    type Result = ResponseActFuture<Self, Result<(), anyhow::Error>>;

    fn handle(&mut self, msg: Dataset, _: &mut Self::Context) -> Self::Result {
        let cache = self.cache.clone();
        async move {
            let conn = with_ctx(|actor: &mut Self, _| actor.db.clone());
            if let Err(err) = conn.send(msg.to_owned()).await {
                log::error!("Failed to persist data in database, Err:{:?}", err);
                return Err(anyhow!(err));
            };

            let mut writer = cache.write().await;
            writer.put(msg.key, msg.data);
            Ok(())
        }
        .interop_actor_boxed(self)
        //self.cache.put(msg.key, msg.data);
    }
}

impl Handler<Key> for KvStore {
    type Result = ResponseActFuture<Self, Result<Vec<u8>, anyhow::Error>>;

    fn handle(&mut self, msg: Key, _: &mut Self::Context) -> Self::Result {
        let cache = self.cache.clone();

        async move {
            let mut c = cache.write().await;

            let d = match c.get(&msg.0) {
                Some(data) => Some(data.to_owned()),
                None => None,
            };
            match d {
                Some(data) => {
                    log::debug!("Cache hit");
                    Ok(data)
                }
                None => {
                    log::warn!("Cache miss");
                    let db = with_ctx(|actor: &mut Self, _| actor.db.clone());
                    if let Ok(Ok(rec)) = db.send(msg.clone()).await {
                        log::debug!("Cache miss, but data found in database");
                        c.put(msg.0, rec.clone());
                        log::debug!("Cached missing data");
                        Ok(rec)
                    } else {
                        log::error!("Key not found in database");
                        Err(anyhow!("Key not found in database"))
                    }
                }
            }
        }
        .interop_actor_boxed(self)
        /*
        match self.cache.get(&msg.0) {
            Some(d) => {
                log::debug!("Cache hit for key: {}", msg.0);
                Ok(d.to_owned())
            }
            None => {
                log::warn!("Cache miss for key: {}", msg.0);
                Err(anyhow!("key not found"))
            }
        }
        */
    }
}
