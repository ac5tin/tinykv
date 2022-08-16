use actix::{Addr, SyncArbiter};
use capnp::capability::Promise;
use futures_util::io::AsyncReadExt;
use std::net::TcpListener;

use capnp_rpc::{pry, rpc_twoparty_capnp, twoparty::VatNetwork, RpcSystem};

use crate::{
    kv::{self, Dataset, KvStore},
    tinykv_capnp,
};

struct TinyKVServer {
    kv: Addr<KvStore>,
}

impl tinykv_capnp::tiny_k_v::Server for TinyKVServer {
    fn set(
        &mut self,
        params: tinykv_capnp::tiny_k_v::SetParams,
        mut results: tinykv_capnp::tiny_k_v::SetResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        let req = pry!(params.get());
        if !req.has_key() || !req.has_value() {
            return Promise::err(capnp::Error::failed("missing key or data".to_owned()));
        }
        let key = req.get_key().unwrap();
        let value = req.get_value().unwrap();

        if self
            .kv
            .try_send(Dataset {
                key: key.to_owned(),
                data: value.to_vec(),
            })
            .is_err()
        {
            return Promise::err(capnp::Error::failed("failed to send message".to_owned()));
        }

        // return values
        results.get().set_key(key);
        results.get().set_value(value);

        // all done
        Promise::ok(())
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
    let tkv = SyncArbiter::start(1, move || kv::KvStore::new());

    let client: tinykv_capnp::tiny_k_v::Client = capnp_rpc::new_client(TinyKVServer { kv: tkv });

    let addr = "0.0.0.0:8321";
    let listener = TcpListener::bind(addr)?;
    log::info!("Listening on {}", addr);

    loop {
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

        let _ = RpcSystem::new(Box::new(network), Some(client.clone().client)).await?;
    }
}
