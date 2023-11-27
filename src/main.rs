//! Remote interface for the Arbitrary waveform generator for Raspberry Pi Pico.
//!
//! This program allows to fully control all settings of the AWG remotely from a
//! computer via USB. It needs the accompanying AWG code to be run on the Pico.
//! Check out the [build instructions] for the hardware setup.
//!
//! # Credits
//!
//! This is a Rust implementation based on the [AWG RemoteUI], originally written in Python
//! by wolf2018 (12-3-2022).
//!
//! [build instructions]: https://www.instructables.com/Poor-Mans-Waveform-Generator-Based-on-RP2040-Raspb/
//! [AWG Remote UI]: https://www.instructables.com/Remote-Control-for-Poor-Mans-Arbitrary-Wave-Form-G/

// Low-level traits
use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
    str::FromStr,
    time::Duration,
};

// Wave generator traits
use pico_wave_gen::{serial::Request, GeneratorFunction, Wave};

// Other traits
use phf::{phf_map, Map};
use serialport::{available_ports, SerialPort};
use slint::{ModelRc, SharedString, VecModel};

slint::include_modules!();

// Modules

mod serial;

/// Default values to ensure proper waveform
static MAX_AMPLITUDE: Map<&'static str, f32> = phf_map! {
    "Sine" => 0.48,
    "Pulse" => 0.89,
    "Gaussian" => 0.55,
    "Sinc" => 0.5,
    "Exponential" => 0.5,
    // "Noise" => 1.0,
};

/// Default values to ensure proper waveform
static OFFSET: Map<&'static str, f32> = phf_map! {
    "Sine" => 0.5,
    "Pulse" => 0.0,
    "Gaussian" => 0.0,
    "Sinc" => 0.5,
    "Exponential" => 0.0,
    // "Noise" => 0.0,
};

/// Possible buffer sizes
/// Have to be a multiple of 4
// enum BufSize {
//     B32 = 32,
//     B64 = 64,
//     B128 = 128,
//     B256 = 256,
//     B512 = 512,
//     B1K = 1024,
//     B2K = 2048,
//     B4K = 4096,
//     B8K = 8_192,
//     B16K = 16_384,
//     B32K = 32_768,
//     B64K = 65_536,
// }

// Setup consts
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;

    // Basic settings
    let ports = match available_ports() {
        Ok(ports) => ports
            .iter()
            .map(|p| SharedString::from(&p.port_name))
            .collect(),
        Err(_) => Vec::from([SharedString::from("No ports found")]),
    };

    let functions =
        ["Sine", "Pulse", "Gaussian", "Sinc", "Exponential"].map(|name| SharedString::from(name));

    // Setup variables
    let wave = Rc::new(RefCell::new(Wave::default()));
    let serial: Rc<RefCell<Option<Box<dyn SerialPort>>>> = Rc::new(RefCell::new(None));
    // let mut serial: Rc<Option<Box<dyn SerialPort>>> = Rc::new(None);

    // Appply initial settings
    ui.set_version(SharedString::from(VERSION));
    ui.set_ports(ModelRc::from(Rc::new(VecModel::from(ports))));
    ui.set_functions(functions.into());

    // Callbacks
    let ui_handle = ui.as_weak();
    let serial_handle = serial.clone();
    ui.on_connect(move |port, connect| {
        let ui = ui_handle.upgrade().unwrap();
        let serial = serial_handle.to_owned();

        match connect {
            true => {
                // AWG is not connected, open port and connect
                let conn = serialport::new(port.as_str(), serial::BAUD_RATE)
                    .timeout(Duration::from_millis(serial::CONN_TIMEOUT))
                    .open();

                ui.set_status(SharedString::from("Waiting for AWG response."));

                match conn {
                    Ok(conn) => {
                        _ = serial.borrow_mut().insert(conn);

                        // Update UI
                        ui.set_status(SharedString::from("Connected to AWG, port: ") + &port);
                        ui.set_status_indicator(true);
                        ui.set_connected(true);
                    }
                    Err(_e) => {
                        // Error establishing connection
                        ui.set_status(SharedString::from("Failed to establish connection to AWG."));
                        ui.set_status_indicator(false);
                    }
                }
            }
            false => {
                // AWG is already connected, close port and disconnect

                // Connection terminated by automatically dropping the value after its dereferenced
                serial.borrow_mut().take();

                // Update UI
                ui.set_status(SharedString::from("AWG Remote UI ready"));
                ui.set_status_indicator(true);
                ui.set_connected(false);
            }
        }
    });

    let ui_handle = ui.as_weak();
    let serial_handle = serial.clone();
    let wave_handle = wave.clone();
    ui.on_setup(move |buf_size| {
        let ui = ui_handle.upgrade().unwrap();
        let serial = serial_handle.to_owned();
        let wave = wave_handle.to_owned();

        // Prepare setup request
        let req = Request {
            command: "setup",
            freq: ui.get_frequency() as u32,
            buf_size: buf_size as u32,
            wave: wave.take(),
        };

        // Send to device
        serial::send_message(req, serial.borrow_mut());

        ui.set_status(SharedString::from("Waiting for AWG response."));

        // Receive status from AWG
        // collect_input()
    });

    let ui_handle = ui.as_weak();
    ui.on_set_function(move |func| {
        let ui = ui_handle.upgrade().unwrap();

        match func.as_str() {
            "Sine" => {
                if !matches!(wave.borrow().func, GeneratorFunction::SINE) {
                    // Initialize parameters
                    let w = wave.borrow_mut();
                    set_defaults(func.as_str(), w, &ui);
                }
            }
            "Pulse" => {
                if !matches!(wave.borrow().func, GeneratorFunction::PULSE) {
                    // Initialize parameters
                    let mut w = wave.borrow_mut();
                    w.set_params([Some(0.05), Some(0.45), Some(0.05)]);
                    set_defaults(func.as_str(), w, &ui);

                    // 	ui.set_rise_time(0.05);
                    // 	ui.set_high_time(0.45);
                    // 	ui.set_fall_time(0.05);
                }
            }
            "Gaussian" => {
                if !matches!(wave.borrow().func, GeneratorFunction::GAUSSIAN) {
                    // Initialize parameters
                    let mut w = wave.borrow_mut();
                    w.set_params([Some(0.095), None, None]);
                    set_defaults(func.as_str(), w, &ui);

                    // 	ui.set_gauss_time(0.095);
                }
            }
            "Sinc" => {
                if !matches!(wave.borrow().func, GeneratorFunction::SINC) {
                    // Initialize parameters
                    let mut w = wave.borrow_mut();
                    w.set_params([Some(0.029), None, None]);
                    set_defaults(func.as_str(), w, &ui);

                    // 	ui.set_sinc_time(0.029);
                }
            }
            "Exponential" => {
                if !matches!(wave.borrow().func, GeneratorFunction::EXPONENTIAL) {
                    // Initialize parameters
                    let mut w = wave.borrow_mut();
                    w.set_params([Some(0.08), None, None]);
                    w.set_replicate(-1);
                    set_defaults(func.as_str(), w, &ui);

                    // 	ui.set_expo_time(0.08);
                }
            }
            _ => {}
        }
    });

    ui.on_stop(|| todo!());

    ui.run()
}

fn set_defaults(func: &str, mut wave: RefMut<'_, Wave>, ui: &MainWindow) {
    wave.func = GeneratorFunction::from_str(func).unwrap();

    let amplitude = *MAX_AMPLITUDE.get(func).unwrap();
    let offset = *OFFSET.get(func).unwrap();

    wave.set_amplitude(amplitude);
    wave.set_offset(offset);
    ui.set_amplitude(amplitude);
    ui.set_offset(offset);
}
