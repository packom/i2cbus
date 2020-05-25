//! Main binary entry point for openapi_client implementation.

use httpd_util::{get_server_addr, https, init_app};

#[path = "server.rs"] mod server;

/// Create custom server, wire it to the autogenerated router,
/// and pass it to the web server.
fn main() {
    init_app(
        "i2cbus",
        "Piers Finlayson, piers@piersandkatie.com",
        "An HTTP(S) microservice exposing I2C bus functionality",
        vec![],
        vec![],
    );

    let addr_socket = get_server_addr();
    let addr_string = format!("{}:{}", addr_socket.ip(), addr_socket.port());
    hyper::rt::run(server::create(&addr_string, https()));
}
