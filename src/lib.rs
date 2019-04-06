//! Main library entry point for openapi_client implementation.
extern crate chrono;
extern crate futures;
extern crate hyper;
extern crate i2cbus_api;
extern crate swagger;
#[macro_use]
extern crate error_chain;
extern crate i2cdev;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

mod http;
mod i2c;
mod server;

mod errors {
    error_chain!{}
}

pub use self::errors::*;
use std::clone::Clone;
use std::io;
use std::marker::PhantomData;
use swagger::{Has, XSpanIdString};

pub struct NewService<C> {
    marker: PhantomData<C>,
}

impl<C> NewService<C> {
    pub fn new() -> Self {
        NewService {
            marker: PhantomData,
        }
    }
}

impl<C> hyper::server::NewService for NewService<C>
where
    C: Has<XSpanIdString> + Clone + 'static,
{
    type Request = (hyper::Request, C);
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Instance = i2cbus_api::server::Service<server::Server<C>, C>;

    /// Instantiate a new server.
    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(i2cbus_api::server::Service::new(server::Server::new()))
    }
}
