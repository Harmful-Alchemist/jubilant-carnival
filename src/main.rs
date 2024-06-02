#![feature(let_chains)]

mod assembly;

use std::ops::BitOr;
use heapless;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::prelude::Peripherals,
    wifi::{AccessPointConfiguration,Configuration, BlockingWifi, EspWifi, Protocol::*, AuthMethod},
};

use assembly::Interpreter;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Start a WiFi AP
    let sysloop = EspSystemEventLoop::take().unwrap();
    let peripherals = Peripherals::take().unwrap();
    let mut esp_wifi = EspWifi::new(peripherals.modem, sysloop.clone(), None).unwrap();
    let ap_config = AccessPointConfiguration {
        ssid: heapless::String::try_from("Connect_to_me").unwrap(),
        ssid_hidden: false,
        channel: 0,
        secondary_channel: Some(1),
        protocols: P802D11B.bitor(P802D11BG).bitor(P802D11BGN).bitor(P802D11BGNLR).bitor(P802D11LR),
        auth_method: AuthMethod::None,
        password: heapless::String::new(),
        max_connections: u16::MAX,
    };
    let config = Configuration::AccessPoint(ap_config);
    esp_wifi.set_configuration(&config).unwrap();
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop).unwrap(); //TODO or non-blocking?
    wifi.start().unwrap();

    let program = vec!["addi x31, x31, 1", "add x31, x31, x31"];

    let interpreter = Interpreter::new(program);

    match interpreter {
        Ok(mut interpreter) => {
            interpreter.step();

            println!("After step 1:\n{:?}\n", interpreter.registers);

            interpreter.step();

            println!("After step 2:\n{:?}\n", interpreter.registers);
        }
        Err(e) => println!("Error:\n{e}\n"),
    }

    loop {}
}
