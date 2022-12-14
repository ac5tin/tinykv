use actix::{Actor, Addr};
use capnp::capability::Promise;
use futures_util::io::AsyncReadExt;
use futures_util::FutureExt;
use tokio::net::TcpListener;

use capnp_rpc::{pry, rpc_twoparty_capnp, twoparty::VatNetwork, RpcSystem};

use crate::{
    db,
    kv::{self, Dataset, Key, KvStore},
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

        let ds = Dataset {
            key: key.to_owned(),
            data: value.to_vec(),
        };

        if self.kv.try_send(ds.clone()).is_err() {
            return Promise::err(capnp::Error::failed("failed to send message".to_owned()));
        };

        // return values
        results.get().set_key(key);
        results.get().set_value(value);

        // all done
        Promise::ok(())
    }

    fn get(
        &mut self,
        params: tinykv_capnp::tiny_k_v::GetParams,
        mut results: tinykv_capnp::tiny_k_v::GetResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        let req = pry!(params.get());
        if !req.has_key() {
            return Promise::err(capnp::Error::failed("missing key".to_owned()));
        }
        let key = req.get_key().unwrap();

        let kkey = key.to_string().clone();
        let req = self.kv.send(Key(key.to_owned()));
        capnp::capability::Promise::from_future(async move {
            if let Ok(Ok(data)) = req.await {
                results.get().set_key(&kkey);
                results.get().set_value(&data.to_vec());
            } else {
                // data doesnt exist, find from db
                // set in cache
                return Err(capnp::Error::failed("failed to get data".to_owned()));
            };
            Ok(())
        })
    }
}

pub async fn start() -> Result<(), anyhow::Error> {
    // init db
    let conn = db::conn::get_conn().await?;
    let tkv = kv::KvStore::new(conn.to_owned()).start();

    let client: tinykv_capnp::tiny_k_v::Client = capnp_rpc::new_client(TinyKVServer { kv: tkv });

    let addr = "0.0.0.0:8321";
    let listener = TcpListener::bind(addr).await?;
    log::info!("Listening on {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let stream = tokio::net::TcpStream::from(stream);
        stream.set_nodelay(true)?;
        let (read_half, write_half) =
            tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();

        let network = VatNetwork::new(
            read_half,
            write_half,
            rpc_twoparty_capnp::Side::Server,
            Default::default(),
        );

        let rpc_system = RpcSystem::new(Box::new(network), Some(client.clone().client));
        tokio::task::spawn_local(Box::pin(rpc_system.map(|_| ()))).await?;
    }
}
