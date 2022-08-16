mod kv;
mod rpc;

pub mod tinykv_capnp {
    include!(concat!(env!("OUT_DIR"), "/tinykv_capnp.rs"));
}

#[actix::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    rpc::server::start().await?;
    Ok(())
}
