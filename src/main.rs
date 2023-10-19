pub mod service;
pub mod subsystem;

// include API
use ws_api::*;

use cubeos_service::{Config, Service,Logger};
use uart_rs::*;
// include output of macro in service.rs file
use crate::service::*;
use crate::subsystem::Subsystem;
// use crate::service::udp_handler;
use failure::*;
use log::{error, info};
use serial::*;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

fn main() -> UartResult<()> {
    Logger::init();
    info!("Start Spiral Blue Service");

    let service_config = Config::new("spiral-blue-service")
        .map_err(|err| {
            error!("Failed to load service config: {:?}", err);
            err
        })
        .unwrap();

    #[cfg(not(any(feature = "ground", feature = "terminal")))]
    let uart_bus = service_config
        .get("uart_bus")
        .ok_or_else(|| {
            error!("Failed to load 'bus' config value");
            format_err!("Failed to load 'bus' config value");
        })
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    // UART Settings can be loaded from the service config file
    // alternatively they can be hardcoded, any change at run time then requires functions in the service
    // let uart_setting = service_config
    // .get("uart_setting")
    // .ok_or_else(|| {
    //     error!("Failed to load 'bus' config value");
    //     format_err!("Failed to load 'bus' config value");
    // })
    // .unwrap();
    #[cfg(not(any(feature = "ground", feature = "terminal")))]
    let uart_setting = serial::PortSettings {
        baud_rate: Baud115200,
        char_size: Bits8,
        parity: ParityNone,
        stop_bits: Stop1,
        flow_control: FlowNone,
    };
    #[cfg(not(any(feature = "ground", feature = "terminal")))]
    let uart_timeout = service_config
        .get("uart_timeout")
        .ok_or_else(|| {
            error!("Failed to load 'bus' config value");
            format_err!("Failed to load 'bus' config value");
        })
        .unwrap();
    #[cfg(not(any(feature = "ground", feature = "terminal")))]
    let uart_timeout: Duration =
        Duration::from_secs(u64::from_str(uart_timeout.as_str().unwrap()).unwrap());

    // Only needed for the ground feature
    #[cfg(any(feature = "ground", feature = "terminal"))]
    let socket = service_config
        .get("udp_socket")
        .ok_or_else(|| {
            error!("Failed to load 'udp-socket' config value");
            format_err!("Failed to load 'udp-socket' config value");
        })
        .unwrap();

    #[cfg(any(feature = "ground", feature = "terminal"))]
    let target = service_config
        .get("target")
        .ok_or_else(|| {
            error!("Failed to load 'target' config value");
            format_err!("Failed to load 'target' config value");
        })
        .unwrap();

    #[cfg(not(any(feature = "ground", feature = "terminal")))]
    let subsystem: Box<Subsystem> = Box::new(
        match Subsystem::new(
            uart_bus,
            uart_setting,
            uart_timeout,
        )
        .map_err(|err| {
            error!("Failed to create subsystem: {:?}", err);
            err
        }) {
            Ok(b) => b,
            Err(e) => {
                info!("Failed to create subsystem");
                panic!("Subsystem creation failed: {:?}", e);
            }
        },
    );

    #[cfg(feature = "debug")]
    service::debug();

    #[cfg(feature = "ground")]
    // Start ground service
    Service::new(
        service_config,
        socket.as_str().unwrap().to_string(),
        target.as_str().unwrap().to_string(),
        Some(Arc::new(json_handler)),
    )
    .start();

    #[cfg(feature = "terminal")]
    // Start ground service
    Service::new(
        service_config,
        socket.as_str().unwrap().to_string(),
        target.as_str().unwrap().to_string(),
        Some(Arc::new(terminal)),
    )
    .start();

    #[cfg(not(any(feature = "ground", feature = "terminal")))]
    //Start up UDP server
    Service::new(service_config, subsystem, Some(Arc::new(udp_handler))).start();

    Ok(())
}
