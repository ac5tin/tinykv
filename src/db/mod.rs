use actix::prelude::*;

use actix_interop::{with_ctx, FutureInterop};
use anyhow::anyhow;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};

use crate::{entity::data, kv::Dataset};

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
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!(e)),
            }
        }
        .interop_actor_boxed(self)
    }
}
