fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("capnp")
        .file("capnp/tinykv.capnp")
        .run()
        .expect("Failed to compile capnproto schema");
}
