//! Serial communication module for remote interface

// Low-level traits
use std::cell::RefMut;

// Wave generator traits
use pico_wave_gen::serial::Request;

// Other traits
use serialport::SerialPort;

// Global configuration
// const TERM_CHAR: &str = "\r\n";
pub const BAUD_RATE: u32 = 115_200;
pub const CONN_TIMEOUT: u64 = 100;

pub fn send_message(req: Request, mut serial: RefMut<'_, Option<Box<dyn SerialPort>>>) {
    let send_msg = serde_json::to_string(&req).unwrap();

    // Send to device
    println!("{}", send_msg);

    // match serial.as_mut() {
    match serial.as_mut() {
        Some(s) => {
            s.write(send_msg.as_bytes()).expect("Write failed!");
        }
        None => todo!(), // Not connected!
    }
}
