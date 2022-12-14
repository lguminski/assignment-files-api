//! Main binary entry point for openapi_lib implementation.

#![allow(missing_docs)]

mod server;


/// Create custom server, wire it to the autogenerated router,
/// and pass it to the web server.
#[tokio::main]
async fn main() {
    env_logger::init();

    let addr = "0.0.0.0:8080";

    server::create(addr, false).await;
}
