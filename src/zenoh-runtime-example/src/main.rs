use std::borrow::Borrow;
use zenoh_runtime_example::{MyParams, MyRuntime};

fn main() {
    println!("Reading runtime configuration...");
    println!("(This is read once at startup from the ZENOH_RUNTIME env var)");

    let app_params: &MyParams = MyRuntime::Application.borrow();
    println!("Application runtime config: {:?}", app_params);

    let net_params: &MyParams = MyRuntime::Network.borrow();
    println!("Network runtime config: {:?}", net_params);

    println!("\nTo test with custom values, run with the environment variable set, for example:");
    println!("ZENOH_RUNTIME='(app: (threads: 10), net: (threads: 1))' cargo run --manifest-path /home/circle/Workings/ZettaScale/src/zenoh/zenoh_runtime_example/Cargo.toml");
}
