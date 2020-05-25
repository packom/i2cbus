use i2cdev2::core::I2CBus;
use i2cdev2::linux::{I2CMsg, LinuxI2CBus, LinuxI2CError};
use std::error::Error;
use std::fmt;
use std::fs::read_dir;
use std::result::Result;

pub(crate) struct BusInfo {
    // ID for this bus, starting at 0
    pub id: usize,

    // Local filesystem path for this bus, likely to be /dev/i2c-<id>
    pub path: String,

    // LinuxI2CBus instance for this bus
    pub bus: LinuxI2CBus,
}

impl BusInfo {
    fn new(id: usize, path: String) -> Result<BusInfo, BusError> {
        let bus = LinuxI2CBus::new(path.clone())?;
        Ok(BusInfo { id, path, bus })
    }

    fn rdwr(&mut self, msgs: &mut Vec<I2CMsg>) -> Result<i32, BusError> {
        self.bus.rdwr(msgs).map_err(From::from)
    }

    // Writes a single byte value to the I2C device with the specified address
    // and to the register.  Is constructed as follows:
    // 1st byte: addr << 1
    // 2nd byte: reg
    // 3rd byte: value
    pub(crate) fn write_reg(&mut self, addr: u16, reg: u8, value: u8) -> Result<i32, BusError> {
        // Build the message
        let mut buf: Vec<u8> = vec![reg, value];
        self.write_bytes(addr, &mut buf)
    }

    // Writes single byte value to the I2C device with the specified address.
    // Is constructed as follows:
    // 1st byte: addr << 1
    // 2nd byte: value
    pub(crate) fn write_byte(&mut self, addr: u16, value: u8) -> Result<i32, BusError> {
        // Build the message
        let mut buf: Vec<u8> = vec![value];
        self.write_bytes(addr, &mut buf)
    }

    // Writes multiple byte value to the I2C device with the specified address.
    // Is constructed as follows:
    // 1st byte: addr << 1
    // 2nd and subsequent byte: values
    pub(crate) fn write_bytes(&mut self, addr: u16, values: &mut Vec<u8>) -> Result<i32, BusError> {
        // Build the message
        let msg = I2CMsg::new(addr, values);
        let mut msgs = vec![msg];

        // Send it
        self.rdwr(&mut msgs).map_err(From::from)
    }

    // Reads from a particular register
    // - First write the register to read from
    // - Then read from it
    pub(crate) fn read_reg(
        &mut self,
        addr: u16,
        reg: u8,
        values: &mut Vec<u8>,
    ) -> Result<i32, BusError> {
        // Need to write the reg then read

        // Build the write message
        let mut bufw: Vec<u8> = vec![reg];
        let msgw = I2CMsg::new(addr, &mut bufw);

        // Build the read message
        let mut msgr = I2CMsg::new(addr, values);
        msgr.set_read();

        let mut msgs = vec![msgw, msgr];

        // Send it
        self.rdwr(&mut msgs).map_err(From::from)
    }

    // Just peforms a read
    // - First write the register to read from
    // - Then read from it
    pub(crate) fn read_bytes(&mut self, addr: u16, values: &mut Vec<u8>) -> Result<i32, BusError> {
        // Build the read message
        let mut msgr = I2CMsg::new(addr, values);
        msgr.set_read();

        let mut msgs = vec![msgr];

        // Send it
        self.rdwr(&mut msgs).map_err(From::from)
    }
}

impl fmt::Display for BusInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "I2C Bus #{} {}", self.id, self.path)
    }
}

#[derive(Debug)]
pub(crate) enum BusError {
    Io(std::io::Error),
    LinuxI2CError(LinuxI2CError),
}

impl fmt::Display for BusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BusError::Io(ref err) => err.fmt(f),
            BusError::LinuxI2CError(ref err) => err.fmt(f),
        }
    }
}

impl Error for BusError {
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            BusError::Io(ref err) => Some(err),
            BusError::LinuxI2CError(ref err) => Some(err),
        }
    }
}

impl From<std::io::Error> for BusError {
    fn from(err: std::io::Error) -> BusError {
        BusError::Io(err)
    }
}

impl From<i2cdev2::linux::LinuxI2CError> for BusError {
    fn from(err: i2cdev2::linux::LinuxI2CError) -> BusError {
        BusError::LinuxI2CError(err)
    }
}

/// Returns an ID and path for each I2C bus found on the system, using the
/// provided directory and I2C bus prefix string
const MAX_BUSES: usize = 127;
pub(crate) fn init_buses(dir_str: &str, prefix_str: &str) -> Result<Vec<BusInfo>, BusError> {
    let mut id: usize = 0;
    let mut buses: Vec<BusInfo> = Vec::new();
    let dir = read_dir(dir_str);
    if let Ok(dir) = dir {
        for entry in dir {
            if let Ok(entry) = entry {
                if let Some(f) = entry.path().file_name() {
                    if let Some(f) = f.to_str() {
                        if f.starts_with(prefix_str) {
                            let path = format!("{}{}", dir_str, f.to_string());
                            match BusInfo::new(id, path.clone()) {
                                Ok(bus) => {
                                    buses.push(bus);
                                    id += 1;
                                    if id >= MAX_BUSES {
                                        println!("Stopped searching for buses - have hit max");
                                        break;
                                    }
                                }
                                Err(e) => println!("Failed to open I2C bus {} {}", path, e),
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(buses)
}
