#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

mod db;
mod entity;
mod kv;
mod rpc;
mod utils;

pub mod tinykv_capnp {
    include!(concat!(env!("OUT_DIR"), "/tinykv_capnp.rs"));
}

#[actix::main]
async fn main() -> Result<(), anyhow::Error> {
    //env_logger::init();
    env_logger::Builder::new()
        .parse_default_env()
        .filter_module("sqlx::query", log::LevelFilter::Off)
        .init();
    rpc::server::start().await?;
    Ok(())
}
