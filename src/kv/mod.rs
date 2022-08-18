use actix::prelude::*;
use actix_interop::{with_ctx, FutureInterop};
use anyhow::anyhow;
use lru::LruCache;
use sea_orm::DatabaseConnection;

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
    cache: LruCache<String, Vec<u8>>,
    db: Addr<DB>,
}

impl KvStore {
    pub fn new(conn: DatabaseConnection) -> KvStore {
        KvStore {
            cache: LruCache::new(100),
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
        if let Err(err) = self.db.try_send(msg.to_owned()) {
            log::error!("Failed to persist data in database, Err:{:?}", err);
            return Err(anyhow!(err));
        };
        self.cache.put(msg.key, msg.data);
        Ok(())
    }
}

impl Handler<Key> for KvStore {
    type Result = ResponseActFuture<Self, Result<Vec<u8>, anyhow::Error>>;

    fn handle(&mut self, msg: Key, _: &mut Self::Context) -> Self::Result {
        async move {
            let d = with_ctx(|actor: &mut Self, _| match actor.cache.get(&msg.0) {
                Some(data) => Some(data.to_owned()),
                None => None,
            });
            match d {
                Some(data) => {
                    log::debug!("Cache hit");
                    Ok(data)
                }
                None => {
                    log::warn!("Cache miss");
                    let db = with_ctx(|actor: &mut Self, _| actor.db.clone());
                    if let Ok(Ok(rec)) = db.send(msg.clone()).await {
                        log::trace!("Cache miss, but data found in database");
                        with_ctx(|actor: &mut Self, _| actor.cache.put(msg.0, rec.clone()));
                        log::trace!("Cached miss data");
                        Ok(rec)
                    } else {
                        Err(anyhow!("Key not found in cache"))
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
