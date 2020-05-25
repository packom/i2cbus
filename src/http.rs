#[path = "i2c.rs"] mod i2c;
use i2cbus_api::models;
use i2cbus_api::{
    I2cBusApiResponse, I2cBusListResponse, I2cBusReadByteResponse, I2cBusReadBytesResponse,
    I2cBusReadRegResponse, I2cBusWriteByteRegResponse, I2cBusWriteByteResponse,
    I2cBusWriteBytesRegResponse, I2cBusWriteBytesResponse,
};
use std::fs;
use std::sync::Mutex;
use lazy_static::lazy_static;
use log::{info, trace, warn};

// Global used to store BUSES - is initialized first time it is used
lazy_static! {
    static ref BUSES: Mutex<Vec<i2c::BusInfo>> = Mutex::new(init_buses());
}

impl<'a> From<&'a i2c::BusInfo> for models::I2cBusList {
    fn from(bus: &i2c::BusInfo) -> Self {
        models::I2cBusList {
            id: Some(bus.id as i32),
            path: Some(bus.path.clone()),
        }
    }
}

// Called to initialize buses with appropriate /dev path
fn init_buses() -> Vec<i2c::BusInfo> {
    const DEV_DIR: &str = "/dev/";
    const I2C_PATH_PREFIX: &str = "i2c-";
    match i2c::init_buses(DEV_DIR, I2C_PATH_PREFIX) {
        Ok(buses) => buses,
        Err(e) => {
            println!("Error calling init_buses {}", e);
            vec![]
        }
    }
}

// Arg errors

enum ArgError {
    Error(models::I2cBusArg),
}

enum ArgErrorType {
    NoSuchBus,
    OutOfBounds,
    NoValues,
}

fn arg_err(arg: &str, val: &str, e_type: &ArgErrorType) -> ArgError {
    let error = match e_type {
        ArgErrorType::NoSuchBus => "no such bus",
        ArgErrorType::OutOfBounds => "out of bounds",
        ArgErrorType::NoValues => "no values",
    };
    ArgError::Error(models::I2cBusArg {
        arg: Some(arg.to_string()),
        description: Some(format!("Invalid value {} ({})", val, error)),
    })
}

// Functions to check arguments passed via HTTP

macro_rules! make_arg_check {
    ($fn_name:ident, $arg:ident, $arg_name:expr, $type:ty, $min:expr, $max:expr) => {
        fn $fn_name(val: &models::$arg) -> Result<$type, ArgError> {
            let val: i32 = val.clone().into();
            if (val <= $max) && (val >= $min) {
                Ok(val as $type)
            } else {
                Err(arg_err(
                    $arg_name,
                    format!("{}", val).as_str(),
                    &ArgErrorType::OutOfBounds,
                ))
            }
        }
    };
}

make_arg_check!(check_arg_addr, Addr, "addr", u16, 0, 255);
make_arg_check!(check_arg_reg, Reg, "reg", u8, 0, 255);
make_arg_check!(check_arg_value, Value, "value", u8, 0, 255);
make_arg_check!(check_arg_num_bytes, NumBytes, "numBytes", u8, 0, 255);
make_arg_check!(check_arg_byte, I2cByte, "I2cByte", u8, 0, 255);

// Construct this function manually
fn check_arg_bus_id(bus_id: &models::BusId) -> Result<usize, ArgError> {
    let buses = BUSES.lock().unwrap();
    let bus_id: i32 = bus_id.clone().into();
    if ((bus_id as usize) < buses.len()) && (bus_id >= 0) {
        Ok(bus_id as usize)
    } else {
        Err(arg_err(
            "busId",
            format!("{}", bus_id).as_str(),
            &ArgErrorType::NoSuchBus,
        ))
    }
}

// Construct this function manually
fn check_arg_values(values: &models::Values) -> Result<Vec<u8>, ArgError> {
    let values = values.clone();
    match values.values {
        Some(v) => {
            let mut rc = Vec::<u8>::with_capacity(v.len());
            for byte in v {
                match check_arg_byte(&byte) {
                    Ok(val) => rc.push(val),
                    Err(_) => {
                        return Err(arg_err(
                            "values",
                            format!("{}", <i32>::from(byte)).as_str(),
                            &ArgErrorType::OutOfBounds,
                        ))
                    }
                }
            }
            Ok(rc)
        }
        None => Err(arg_err("values", "{{}}", &ArgErrorType::NoValues)),
    }
}

fn write_byte_reg_check_args(
    bus_id: &models::BusId,
    addr: &models::Addr,
    reg: &models::Reg,
    value: &models::Value,
) -> Result<(usize, u16, u8, u8), ArgError> {
    let bus_id = check_arg_bus_id(&bus_id)?;
    let addr = check_arg_addr(&addr)?;
    let reg = check_arg_reg(&reg)?;
    let value = check_arg_value(&value)?;
    Ok((bus_id, addr, reg, value))
}

fn write_byte_check_args(
    bus_id: &models::BusId,
    addr: &models::Addr,
    value: &models::Value,
) -> Result<(usize, u16, u8), ArgError> {
    let bus_id = check_arg_bus_id(&bus_id)?;
    let addr = check_arg_addr(&addr)?;
    let value = check_arg_value(&value)?;
    Ok((bus_id, addr, value))
}

fn write_bytes_check_args(
    bus_id: &models::BusId,
    addr: &models::Addr,
    values: &models::Values,
) -> Result<(usize, u16, Vec<u8>), ArgError> {
    let bus_id = check_arg_bus_id(&bus_id)?;
    let addr = check_arg_addr(&addr)?;
    let values = check_arg_values(&values)?;
    Ok((bus_id, addr, values))
}

fn write_bytes_reg_check_args(
    bus_id: &models::BusId,
    addr: &models::Addr,
    reg: &models::Reg,
    values: &models::Values,
) -> Result<(usize, u16, u8, Vec<u8>), ArgError> {
    let bus_id = check_arg_bus_id(&bus_id)?;
    let addr = check_arg_addr(&addr)?;
    let reg = check_arg_reg(&reg)?;
    let values = check_arg_values(&values)?;
    Ok((bus_id, addr, reg, values))
}

fn read_reg_check_args(
    bus_id: &models::BusId,
    addr: &models::Addr,
    reg: &models::Reg,
    num_bytes: &models::NumBytes,
) -> Result<(usize, u16, u8, u8), ArgError> {
    let bus_id = check_arg_bus_id(&bus_id)?;
    let addr = check_arg_addr(&addr)?;
    let reg = check_arg_reg(&reg)?;
    let num_bytes = check_arg_num_bytes(&num_bytes)?;
    Ok((bus_id, addr, reg, num_bytes))
}

fn read_byte_check_args(
    bus_id: &models::BusId,
    addr: &models::Addr,
) -> Result<(usize, u16), ArgError> {
    let bus_id = check_arg_bus_id(&bus_id)?;
    let addr = check_arg_addr(&addr)?;
    Ok((bus_id, addr))
}

fn read_bytes_check_args(
    bus_id: &models::BusId,
    addr: &models::Addr,
    num_bytes: &models::NumBytes,
) -> Result<(usize, u16, u8), ArgError> {
    let bus_id = check_arg_bus_id(&bus_id)?;
    let addr = check_arg_addr(&addr)?;
    let num_bytes = check_arg_num_bytes(&num_bytes)?;
    Ok((bus_id, addr, num_bytes))
}

macro_rules! impl_from_arg_error {
    ($type:tt) => {
        impl From<ArgError> for $type {
            fn from(e: ArgError) -> Self {
                let ArgError::Error(e) = e;
                $type::BadRequest(e)
            }
        }
    };
}

impl_from_arg_error!(I2cBusWriteByteRegResponse);
impl_from_arg_error!(I2cBusWriteByteResponse);
impl_from_arg_error!(I2cBusWriteBytesResponse);
impl_from_arg_error!(I2cBusWriteBytesRegResponse);
impl_from_arg_error!(I2cBusReadRegResponse);
impl_from_arg_error!(I2cBusReadByteResponse);
impl_from_arg_error!(I2cBusReadBytesResponse);

macro_rules! unwrap_or_return_rsp {
    ($fn:tt, $exp:expr) => {
        match $exp {
            Ok(x) => x,
            Err(e) => {
                let rsp = e.into();
                info!("API {} -> {:?}", stringify!($fn), rsp);
                return rsp
            },
        }
    };
}

macro_rules! impl_from_i2c_bus_error {
    ($type:tt) => {
        impl From<i2c::BusError> for $type {
            fn from(e: i2c::BusError) -> Self {
                match e {
                    i2c::BusError::LinuxI2CError(e) => match e {
                        i2cdev2::linux::LinuxI2CError::Nix(e) => match e {
                            nix::Error::Sys(e) => $type::TransactionFailed(models::I2cBusError {
                                error: Some(e as i32),
                                description: Some(format!("{:?}", e)),
                            }),
                            _ => $type::TransactionFailed(models::I2cBusError {
                                error: None,
                                description: Some(format!("{:?}", e)),
                            }),
                        },
                        i2cdev2::linux::LinuxI2CError::Io(e) => {
                            $type::TransactionFailed(models::I2cBusError {
                                error: Some(e.raw_os_error().unwrap()),
                                description: Some(format!("{:?}", e)),
                            })
                        }
                    },
                    // Can't be hit - only used in init_buses()
                    i2c::BusError::Io(e) => $type::TransactionFailed(models::I2cBusError {
                        error: Some(e.raw_os_error().unwrap()),
                        description: Some(format!("{:?}", e)),
                    }),
                }
            }
        }
    };
}

impl_from_i2c_bus_error!(I2cBusWriteByteRegResponse);
impl_from_i2c_bus_error!(I2cBusWriteBytesRegResponse);
impl_from_i2c_bus_error!(I2cBusWriteByteResponse);
impl_from_i2c_bus_error!(I2cBusWriteBytesResponse);
impl_from_i2c_bus_error!(I2cBusReadRegResponse);
impl_from_i2c_bus_error!(I2cBusReadByteResponse);
impl_from_i2c_bus_error!(I2cBusReadBytesResponse);

pub(crate) fn write_byte(
    bus_id: &models::BusId,
    addr: &models::Addr,
    value: &models::Value,
) -> I2cBusWriteByteResponse {
    info!("API {} : {:?} {:?} {:?}", "write_byte", bus_id, addr, value);
    let (bus_id, addr, value) =
        unwrap_or_return_rsp!(write_byte, write_byte_check_args(&bus_id, &addr, &value));
    let mut buses = BUSES.lock().unwrap();
    let rsp = match buses[bus_id].write_byte(addr, value) {
        Ok(rc) => I2cBusWriteByteResponse::OK(models::I2cBusOk { ok: Some(rc) }),
        Err(e) => e.into(),
    };
    info!("API {} -> {:?}", "write_byte", rsp);
    rsp
}

pub(crate) fn write_bytes(
    bus_id: &models::BusId,
    addr: &models::Addr,
    values: &models::Values,
) -> I2cBusWriteBytesResponse {
    info!("API {} : {:?} {:?} {:?}", "write_bytes", bus_id, addr, values);
    let (bus_id, addr, mut values) =
        unwrap_or_return_rsp!(write_bytes, write_bytes_check_args(&bus_id, &addr, &values));
    let mut buses = BUSES.lock().unwrap();
    let rsp = match buses[bus_id].write_bytes(addr, &mut values) {
        Ok(rc) => I2cBusWriteBytesResponse::OK(models::I2cBusOk { ok: Some(rc) }),
        Err(e) => e.into(),
    };
    info!("API {} -> {:?}", "write_bytes", rsp);
    rsp
}

pub(crate) fn write_bytes_reg(
    bus_id: &models::BusId,
    addr: &models::Addr,
    reg: &models::Reg,
    values: &models::Values,
) -> I2cBusWriteBytesRegResponse {
    info!("API {} : {:?} {:?} {:?} {:?}", "write_bytes_reg", bus_id, addr, reg, values);
    let (bus_id, addr, reg, mut values) =
        unwrap_or_return_rsp!(write_bytes_reg, write_bytes_reg_check_args(&bus_id, &addr, &reg, &values));
    values.insert(0, reg);
    let mut buses = BUSES.lock().unwrap();
    let rsp = match buses[bus_id].write_bytes(addr, &mut values) {
        Ok(rc) => I2cBusWriteBytesRegResponse::OK(models::I2cBusOk { ok: Some(rc) }),
        Err(e) => e.into(),
    };
    info!("API {} -> {:?}", "write_bytes_reg", rsp);
    rsp
}

pub(crate) fn write_byte_reg(
    bus_id: &models::BusId,
    addr: &models::Addr,
    reg: &models::Reg,
    value: &models::Value,
) -> I2cBusWriteByteRegResponse {
    info!("API {} : {:?} {:?} {:?} {:?}", "write_byte_reg", bus_id, addr, reg, value);
    let (bus_id, addr, reg, value) =
        unwrap_or_return_rsp!(write_byte_reg, write_byte_reg_check_args(&bus_id, &addr, &reg, &value));
    let mut buses = BUSES.lock().unwrap();
    let rsp = match buses[bus_id].write_reg(addr, reg, value) {
        Ok(rc) => I2cBusWriteByteRegResponse::OK(models::I2cBusOk { ok: Some(rc) }),
        Err(e) => e.into(),
    };
    info!("API {} -> {:?}", "write_byte_reg", rsp);
    rsp
}

pub(crate) fn read_byte(bus_id: &models::BusId, addr: &models::Addr) -> I2cBusReadByteResponse {
    info!("API {} : {:?} {:?}", "read_byte", bus_id, addr);
    let (bus_id, addr) = unwrap_or_return_rsp!(read_byte, read_byte_check_args(&bus_id, &addr));
    let mut buses = BUSES.lock().unwrap();
    let mut values: Vec<u8> = vec![0; 1];
    let rsp = match buses[bus_id].read_bytes(addr, &mut values) {
        Ok(rc) => I2cBusReadByteResponse::OK(models::I2cBusRead {
            ok: Some(rc),
            values: {
                Some(
                    values
                        .iter()
                        .map(|x| <i32>::try_from(*x).unwrap())
                        .map(<models::I2cByte>::from)
                        .collect::<Vec<models::I2cByte>>(),
                )
            },
        }),
        Err(e) => e.into(),
    };
    info!("API {} -> {:?}", "read_byte", rsp);
    rsp
}

use std::convert::TryFrom;
pub(crate) fn read_bytes(
    bus_id: &models::BusId,
    addr: &models::Addr,
    num_bytes: &models::NumBytes,
) -> I2cBusReadBytesResponse {
    info!("API {} : {:?} {:?} {:?}", "read_bytes", bus_id, addr, num_bytes);
    let (bus_id, addr, num_bytes) =
        unwrap_or_return_rsp!(read_bytes, read_bytes_check_args(&bus_id, &addr, &num_bytes));
    let mut buses = BUSES.lock().unwrap();
    let mut values: Vec<u8> = vec![0; num_bytes as usize];
    let rsp = match buses[bus_id].read_bytes(addr, &mut values) {
        Ok(rc) => I2cBusReadBytesResponse::OK(models::I2cBusRead {
            ok: Some(rc),
            values: {
                Some(
                    values
                        .iter()
                        .map(|x| <i32>::try_from(*x).unwrap())
                        .map(<models::I2cByte>::from)
                        .collect::<Vec<models::I2cByte>>(),
                )
            },
        }),
        Err(e) => e.into(),
    };
    info!("API {} -> {:?}", "read_bytes", rsp);
    rsp
}

pub(crate) fn read_reg(
    bus_id: &models::BusId,
    addr: &models::Addr,
    reg: &models::Reg,
    num_bytes: &models::NumBytes,
) -> I2cBusReadRegResponse {
    info!("API {} : {:?} {:?} {:?} {:?}", "read_reg", bus_id, addr, reg, num_bytes);
    let (bus_id, addr, reg, num_bytes) =
        unwrap_or_return_rsp!(read_reg, read_reg_check_args(&bus_id, &addr, &reg, &num_bytes));
    let mut buses = BUSES.lock().unwrap();
    let mut values: Vec<u8> = vec![0; num_bytes as usize];
    let rsp = match buses[bus_id].read_reg(addr, reg, &mut values) {
        Ok(rc) => I2cBusReadRegResponse::OK(models::I2cBusRead {
            ok: Some(rc),
            values: {
                Some(
                    values
                        .iter()
                        .map(|x| <i32>::try_from(*x).unwrap())
                        .map(<models::I2cByte>::from)
                        .collect::<Vec<models::I2cByte>>(),
                )
            },
        }),
        Err(e) => e.into(),
    };
    info!("API {} -> {:?}", "read_reg", rsp);
    rsp
}

pub(crate) fn get_api() -> I2cBusApiResponse {
    // Read in the file
    info!("API {}", "get_api");
    let rsp = match fs::read("/static/api.yaml") {
        Ok(api) => match String::from_utf8(api) {
            Ok(s) => I2cBusApiResponse::OK(s),
            Err(e) => I2cBusApiResponse::FileNotFound(
                format!("Hit error parsing API file {}", e).to_string(),
            ),
        },
        Err(e) => I2cBusApiResponse::FileNotFound(
            format!("File not found {}", e).to_string(),
        ),
    };
    info!("API {} -> {:?}", "get_api", rsp);
    rsp
}

pub(crate) fn get_buses() -> I2cBusListResponse {
    let buses = BUSES.lock().unwrap();
    info!("API {}", "get_buses");
    let rsp = I2cBusListResponse::OK(
        buses
            .iter()
            .map(<models::I2cBusList>::from)
            .collect::<Vec<models::I2cBusList>>(),
    );
    info!("API {} -> {:?}", "get_buses", rsp);
    rsp
}

