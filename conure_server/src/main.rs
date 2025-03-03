mod config;

pub mod conure_rpc_capnp {
    include!(concat!(env!("OUT_DIR"), "/conure_rpc_capnp.rs"));
}

fn main() {
    env_logger::init();

    println!("Hello, world!");
}
