use futures_util::io::AsyncReadExt;
use std::net::TcpListener;

use capnp_rpc::{rpc_twoparty_capnp, twoparty::VatNetwork, RpcSystem};

use crate::tinykv_capnp;

struct TinyKVServer;

impl tinykv_capnp::tiny_k_v::Server for TinyKVServer {
    fn set(
        &mut self,
        _: tinykv_capnp::tiny_k_v::SetParams,
        _: tinykv_capnp::tiny_k_v::SetResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        capnp::capability::Promise::err(capnp::Error::unimplemented(
            "method tiny_k_v::Server::set not implemented".to_string(),
        ))
    }

    fn get(
        &mut self,
        _: tinykv_capnp::tiny_k_v::GetParams,
        _: tinykv_capnp::tiny_k_v::GetResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        capnp::capability::Promise::err(capnp::Error::unimplemented(
            "method tiny_k_v::Server::get not implemented".to_string(),
        ))
    }
}

pub async fn start() -> Result<(), anyhow::Error> {
    let addr = "0.0.0.0:8321";
    let listener = TcpListener::bind(addr)?;
    log::info!("Listening on {}", addr);
    let (stream, _) = listener.accept()?;
    stream.set_nonblocking(true)?;
    let stream = tokio::net::TcpStream::from_std(stream)?;
    let (read_half, write_half) =
        tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();

    let network = VatNetwork::new(
        read_half,
        write_half,
        rpc_twoparty_capnp::Side::Server,
        Default::default(),
    );

    let client: tinykv_capnp::tiny_k_v::Client = capnp_rpc::new_client(TinyKVServer);

    let _ = RpcSystem::new(Box::new(network), Some(client.clone().client)).await?;

    Ok(())
}
