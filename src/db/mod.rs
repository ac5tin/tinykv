use actix::prelude::*;

use actix_interop::{with_ctx, FutureInterop};
use anyhow::anyhow;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::{entity::data, kv::Dataset, kv::Key};

pub mod conn;

pub struct DB {
    conn: DatabaseConnection,
}

impl DB {
    pub fn new(conn: DatabaseConnection) -> DB {
        DB { conn }
    }
}

impl Actor for DB {
    type Context = Context<Self>;
}

impl Handler<Dataset> for DB {
    type Result = ResponseActFuture<Self, Result<(), anyhow::Error>>;

    fn handle(&mut self, msg: Dataset, _: &mut Context<Self>) -> Self::Result {
        let d = data::ActiveModel {
            key: Set(msg.key), // use Set() to raw value convert to ActiveValue
            value: Set(msg.data),
            ..Default::default()
        };

        async move {
            let conn = with_ctx(|actor: &mut Self, _| actor.conn.clone());
            match d.insert(&conn).await {
                Ok(_) => {
                    log::debug!("Insert data to database successfully");
                    Ok(())
                }
                Err(e) => {
                    if e != sea_orm::error::DbErr::Exec("error returned from database: (code: 2067) UNIQUE constraint failed: data.key".to_owned()) {
                        log::error!("Failed to insert data to database, Err:{:?}", e);
                    };
                    Err(anyhow!(e))
                }
            }
        }
        .interop_actor_boxed(self)
    }
}

impl Handler<Key> for DB {
    type Result = ResponseActFuture<Self, Result<Vec<u8>, anyhow::Error>>;

    fn handle(&mut self, msg: Key, _: &mut Context<Self>) -> Self::Result {
        async move {
            let conn = with_ctx(|actor: &mut Self, _| actor.conn.clone());
            match data::Entity::find()
                .filter(data::Column::Key.eq(msg.0))
                .one(&conn)
                .await
            {
                Ok(Some(d)) => Ok(d.value),
                Ok(None) => Err(anyhow!("Key not found in database")),
                Err(e) => Err(anyhow!(e)),
            }
        }
        .interop_actor_boxed(self)
    }
}
