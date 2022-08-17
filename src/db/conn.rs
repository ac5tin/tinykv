use migration::{Migrator, MigratorTrait};
use sea_orm::DatabaseConnection;

use crate::utils;

pub async fn get_conn() -> Result<DatabaseConnection, anyhow::Error> {
    utils::fs::create_if_not_exist("./data.db")?;
    let conn = sea_orm::Database::connect("sqlite://data.db").await?;
    Migrator::up(&conn, None).await?;
    Ok(conn)
}
