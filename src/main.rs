#![feature(let_chains)]

mod assembly;

use std::ops::BitOr;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::prelude::Peripherals,
    wifi::{
        AccessPointConfiguration, AuthMethod, BlockingWifi, Configuration, EspWifi, Protocol::*,
    },
};
use heapless;

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
        protocols: P802D11B
            .bitor(P802D11BG)
            .bitor(P802D11BGN)
            .bitor(P802D11BGNLR)
            .bitor(P802D11LR),
        auth_method: AuthMethod::None,
        password: heapless::String::new(),
        max_connections: u16::MAX,
    };
    let config = Configuration::AccessPoint(ap_config);
    esp_wifi.set_configuration(&config).unwrap();
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop).unwrap(); //TODO or non-blocking?
    wifi.start().unwrap();

    let listener = TcpListener::bind("0.0.0.0:80").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }

    fn handle_connection(mut stream: TcpStream) {
        let buf_reader = BufReader::new(&mut stream);
        let http_request: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        let program = vec!["addi x31, x31, 1", "add x31, x31, x31"];
        let mut interpreter = Interpreter::new(program).unwrap();

        interpreter.step();

        println!("After step 1:\n{:?}\n", interpreter.registers);

        interpreter.step();

        println!("After step 2:\n{:?}\n", interpreter.registers);

        let status_line = "HTTP/1.1 200 OK";
        let contents = format!(
            r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>Hello!</title>
  </head>
  <body>
    <h1>Hello!</h1>
    <p>Hi from Rust on the ESP32</p>
    <p>{:?}<p>
  </body>
</html>"#,
            interpreter.registers
        );
        let length = contents.len();

        let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

        stream.write_all(response.as_bytes()).unwrap();
    }
}
