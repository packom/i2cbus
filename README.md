# i2cbus

A RESTful HTTP microservice exposing I2C bus control.

## Building

```
git clone https://github.com/packom/i2cbus
cd i2cbus
cargo build
```

## Running

i2cbus uses environment variables for configuration, as it's intended to be run within a container.

To run bound to localhost:8080 with INFO level logging:

```
env SERVER_IP=localhost \
env SERVER_PORT=8080 \
env RUST_LOG=INFO \
cargo run
```

Use environment variable HTTPS (no value is necessary) to enable HTTPS support, e.g.:

```
env SERVER_IP=localhost \
env SERVER_PORT=8443 \
env HTTPS= \
env RUST_LOG=INFO \
cargo run
```

i2cbus expects to find certificate and key files at the following paths (which are not currently configurable):

```
/ssl/key.pem
/ssl/cert.pem
```

To see other options run:

```
cargo run -- --help
```

## Controlling the I2C bus

To see examples controlling the I2C bus see [here](https://github.com/packom/i2cbus/blob/master/notes/examples.txt).