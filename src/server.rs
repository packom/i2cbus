//! Server implementation of openapi_client.
#![allow(unused_imports)]

extern crate i2cbus_api;
extern crate swagger;

use chrono;
use futures::{self, Future};
use std::collections::HashMap;
use std::marker::PhantomData;

//use swagger;
use swagger::{Has, XSpanIdString};

use i2cbus_api::models;
use i2cbus_api::{
    Api, ApiError, I2cBusApiResponse, I2cBusListResponse, I2cBusReadByteResponse,
    I2cBusReadBytesResponse, I2cBusReadRegResponse, I2cBusWriteByteRegResponse,
    I2cBusWriteByteResponse, I2cBusWriteBytesRegResponse, I2cBusWriteBytesResponse,
};

use crate::http;

#[derive(Copy, Clone)]
pub struct Server<C> {
    marker: PhantomData<C>,
}

impl<C> Server<C> {
    pub fn new() -> Self {
        Server {
            marker: PhantomData,
        }
    }
}

impl<C> Api<C> for Server<C>
where
    C: Has<XSpanIdString>,
{
    fn i2c_bus_api(&self, _context: &C) -> Box<Future<Item = I2cBusApiResponse, Error = ApiError>> {
        Box::new(futures::future::ok(http::get_api()))
    }

    fn i2c_bus_list(
        &self,
        _context: &C,
    ) -> Box<Future<Item = I2cBusListResponse, Error = ApiError>> {
        Box::new(futures::future::ok(http::get_buses()))
    }

    fn i2c_bus_read_byte(
        &self,
        bus_id: models::BusId,
        addr: models::Addr,
        _context: &C,
    ) -> Box<Future<Item = I2cBusReadByteResponse, Error = ApiError>> {
        Box::new(futures::future::ok(http::read_byte(&bus_id, &addr)))
    }

    fn i2c_bus_read_bytes(
        &self,
        bus_id: models::BusId,
        addr: models::Addr,
        num_bytes: models::NumBytes,
        _context: &C,
    ) -> Box<Future<Item = I2cBusReadBytesResponse, Error = ApiError>> {
        Box::new(futures::future::ok(http::read_bytes(
            &bus_id, &addr, &num_bytes,
        )))
    }

    fn i2c_bus_read_reg(
        &self,
        bus_id: models::BusId,
        addr: models::Addr,
        reg: models::Reg,
        num_bytes: models::NumBytes,
        _context: &C,
    ) -> Box<Future<Item = I2cBusReadRegResponse, Error = ApiError>> {
        Box::new(futures::future::ok(http::read_reg(
            &bus_id, &addr, &reg, &num_bytes,
        )))
    }

    fn i2c_bus_write_byte(
        &self,
        bus_id: models::BusId,
        addr: models::Addr,
        value: models::Value,
        _context: &C,
    ) -> Box<Future<Item = I2cBusWriteByteResponse, Error = ApiError>> {
        Box::new(futures::future::ok(http::write_byte(
            &bus_id, &addr, &value,
        )))
    }

    fn i2c_bus_write_bytes(
        &self,
        bus_id: models::BusId,
        addr: models::Addr,
        values: models::Values,
        _context: &C,
    ) -> Box<Future<Item = I2cBusWriteBytesResponse, Error = ApiError>> {
        Box::new(futures::future::ok(http::write_bytes(
            &bus_id, &addr, &values,
        )))
    }

    fn i2c_bus_write_byte_reg(
        &self,
        bus_id: models::BusId,
        addr: models::Addr,
        reg: models::Reg,
        value: models::Value,
        _context: &C,
    ) -> Box<Future<Item = I2cBusWriteByteRegResponse, Error = ApiError>> {
        Box::new(futures::future::ok(http::write_byte_reg(
            &bus_id, &addr, &reg, &value,
        )))
    }

    fn i2c_bus_write_bytes_reg(
        &self,
        bus_id: models::BusId,
        addr: models::Addr,
        reg: models::Reg,
        values: models::Values,
        _context: &C,
    ) -> Box<Future<Item = I2cBusWriteBytesRegResponse, Error = ApiError>> {
        Box::new(futures::future::ok(http::write_bytes_reg(
            &bus_id, &addr, &reg, &values,
        )))
    }
}
