use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, DatabaseConnection};

use crate::utils;

pub async fn get_conn() -> Result<DatabaseConnection, anyhow::Error> {
    utils::fs::create_if_not_exist("./data.db")?;
    // bad practise to use multiple connections when connecting to sqlite
    let mut opts = ConnectOptions::new("sqlite://data.db".to_owned());
    opts.sqlx_logging(false); // disable sqlx logging
    let conn = sea_orm::Database::connect(opts).await?;
    Migrator::up(&conn, None).await?;
    Ok(conn)
}
