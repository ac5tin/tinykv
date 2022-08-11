mod rpc;

pub mod tinykv_capnp {
    include!(concat!(env!("OUT_DIR"), "/tinykv_capnp.rs"));
}

fn main() {
    println!("Hello, world!");
}
