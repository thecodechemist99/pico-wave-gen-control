//! Serial communication module for remote interface

// Low-level traits
use std::{
    cell::RefMut,
    io::{Error, ErrorKind},
};

// Wave generator traits
use pico_wave_gen::serial::Request;

// Other traits
use serialport::{available_ports, SerialPort, SerialPortInfo, SerialPortType};

// Global configuration
// const TERM_CHAR: &str = "\r\n";
pub const BAUD_RATE: u32 = 115_200;
pub const CONN_TIMEOUT: u64 = 100;

pub fn get_ports() -> Vec<SerialPortInfo> {
    match available_ports() {
        Ok(ports) => ports,
        Err(_) => vec![SerialPortInfo {
            port_name: String::from("No ports found"),
            port_type: SerialPortType::Unknown,
        }],
    }
}

/// Sends request as JSON string to the attached serial device.
pub fn send_request(
    req: Request,
    mut serial: RefMut<'_, Option<Box<dyn SerialPort>>>,
) -> Result<(), Error> {
    let send_msg = serde_json::to_string(&req).unwrap();

    // Send to device
    println!("{}", send_msg);

    match serial.as_mut() {
        Some(s) => {
            // Check if message buffer is larger than 64 Bytes and split if necessary
            let chunks = send_msg.as_bytes().chunks(64);
            for chunk in chunks {
                let written_bytes = s.write(&chunk);
                println!("{:?}", written_bytes);
            }
            Ok(())
        }
        None => Err(Error::new(
            ErrorKind::NotConnected,
            "Not connected to a device",
        )),
    }
}
