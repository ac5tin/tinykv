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
