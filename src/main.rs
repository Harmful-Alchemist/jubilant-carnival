#![feature(let_chains)]

mod assembly;

use std::cell::RefCell;
use std::ops::BitOr;
use std::rc::Rc;
use std::vec::Vec;
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

use assembly::Interpreter;

type Interpreters = Rc<RefCell<(u8, [Interpreter; 5])>>;

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

    let interpreters: Interpreters = Rc::new(RefCell::new((
        0,
        [
            Interpreter::new(vec![]).unwrap(),
            Interpreter::new(vec![]).unwrap(),
            Interpreter::new(vec![]).unwrap(),
            Interpreter::new(vec![]).unwrap(),
            Interpreter::new(vec![]).unwrap(),
        ],
    )));
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream, interpreters.clone());
    }

    fn handle_connection(mut stream: TcpStream, interpreters: Interpreters) {
        let buf_reader = BufReader::new(&mut stream);

        let mut lines = buf_reader.lines();

        let line = lines.next().unwrap().unwrap();

        let status_line = "HTTP/1.1 200 OK";

        let response;
        if line.contains("GET") {
            response = get_resp();
        } else if line.contains("/new") && line.contains("POST") {
            let mut registers = Vec::new();
            let mut program = Vec::new();
            while let Some(Ok(line)) = lines.next() {
                if line.contains("###") {
                    break;
                }
                if !registers.is_empty() && !line.trim().is_empty() {
                    program.push(line.trim().to_string());
                }
                if line.starts_with("[") {
                    registers = line
                        .replace("[", "")
                        .replace("]", "")
                        .split(',')
                        .map(|s| s.trim().parse::<i32>().unwrap_or(0))
                        .collect();
                };
            }

            let interpreter = Interpreter::new(program.to_vec());

            let contents = match interpreter {
                Ok(mut interpreter) => {
                    for i in 4..=6 {
                        interpreter.registers[i] = registers[i];
                    }
                    for i in 27..=30 {
                        interpreter.registers[i] = registers[i];
                    }
                    let mut interpreters = interpreters.borrow_mut();
                    let i = interpreters.0;
                    let contents = step(&mut interpreter, i + 1);
                    interpreters.1[i as usize] = interpreter;
                    interpreters.0 += (interpreters.0 + 1) % 5;
                    contents
                }
                Err(e) => {
                    format!(r#"{{"error":"{e}"}}"#)
                }
            };

            let length = contents.len();
            response = format!("{status_line}\r\nContent-Length: {length}\r\nContent-Type: application/json\r\n\r\n{contents}");
        } else if line.contains("POST") {
            match line.split(" ").collect::<Vec<_>>()[1]
                .replace("/", "")
                .parse::<u8>()
            {
                Err(_) => response = not_found(),
                Ok(x) if x > 5 => response = not_found(),
                Ok(x) => {
                    let mut interpreters = interpreters.borrow_mut(); //TODO panice, multiple but whatever one user, me, now
                    let interpreter: &mut Interpreter = &mut interpreters.1[(x - 1) as usize];
                    let contents = step(interpreter, x);

                    let length = contents.len();
                    response = format!("{status_line}\r\nContent-Length: {length}\r\nContent-Type: application/json\r\n\r\n{contents}");
                }
            };
        } else {
            response = not_found();
        }
        stream.write_all(response.as_bytes()).unwrap();
    }
}

fn step(interpreter: &mut Interpreter, program_number: u8) -> String {
    let line = interpreter.line + 1; //Send the line we are going to execute
    match interpreter.step() {
        Some(_) => {
            let registers = interpreter.registers;
            format!(
                r#"{{"line":{line},"registers":{registers:?},"program_number":{program_number}}}"#
            )
        }
        None => r#"{"done":true}"#.to_string(),
    }
}

fn not_found() -> String {
    let status_line = "HTTP/1.1 404 NOT FOUND";
    format!("{status_line}\r\nContent-Length: 0\r\n\r\n")
}

fn get_resp() -> String {
    let status_line = "HTTP/1.1 200 OK";

    let contents = r#"<!doctype html>
    <html lang="en-US">
    
    <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width" />
        <title>RISC-V executor</title>
        <style>
            .error {
                background-color: red;
                visibility: hidden;
            }
    
            .container {
                display: flex;
            }
    
            /* Thanks https://webtips.dev/add-line-numbers-to-html-textarea ! */
            .editor {
                display: inline-flex;
                gap: 10px;
                font-family: monospace;
                line-height: 21px;
                border-radius: 2px;
                padding: 20px 10px;
                overflow: auto;
            }
    
            textarea {
                line-height: 21px;
                overflow-y: hidden;
                padding: 0;
                border: 0;
                min-width: 500px;
                outline: none;
                resize: none;
            }
    
            .line-numbers {
                width: 20px;
                text-align: right;
            }
    
            .line-numbers span {
                counter-increment: linenumber;
            }
    
            .line-numbers span::before {
                content: counter(linenumber);
                display: block;
            }
    
            .line-numbers .executed::before {
                background-color: red;
            }
        </style>
    </head>
    
    <body>
        <div id="error" class="error">
            <p>Errors here</p>
        </div>
    
        <div class="container">
            <label for="program">Enter your assembly here:</label>
    
            <div class="editor">
                <div class="line-numbers">
                    <span class="line-number"></span>
                </div>
                <textarea id="program" name="program" rows="20" cols="33">
    addi x5, x5, 10
            </textarea>
            </div>
    
            <button onclick="step()">Step</button>
    
            <button onclick="reset_code()">Reset code</button>
    
            <table>
                <tr>
                    <th>Register</th>
                    <th>Alternative name</th>
                    <th>Value</th>
                </tr>
                <tr>
                    <td>x0</td>
                    <td>zero</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x1</td>
                    <td>ra</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x2</td>
                    <td>sp</td>
                    <td>Can't read</td>
                </tr>
                <tr>
                    <td>x3</td>
                    <td>gp</td>
                    <td>Can't read</td>
                </tr>
                <tr>
                    <td>x4</td>
                    <td>tp</td>
                    <td>Can't read</td>
                </tr>
                <tr>
                    <td>x5</td>
                    <td>t0</td>
                    <td><input id="x5" type="number" value="0" /></td>
                </tr>
                <tr>
                    <td>x6</td>
                    <td>t1</td>
                    <td><input id="x6" type="number" value="0" /></td>
                </tr>
                <tr>
                    <td>x7</td>
                    <td>t2</td>
                    <td><input id="x7" type="number" value="0" /></td>
                </tr>
                <tr>
                    <td>x8</td>
                    <td>s0/fp</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x9</td>
                    <td>s1</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x10</td>
                    <td>a0</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x11</td>
                    <td>a1</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x12</td>
                    <td>a2</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x13</td>
                    <td>a3</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x14</td>
                    <td>a4</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x15</td>
                    <td>a5</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x16</td>
                    <td>a6</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x17</td>
                    <td>a7</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x18</td>
                    <td>s2</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x19</td>
                    <td>s3</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x20</td>
                    <td>s4</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x21</td>
                    <td>s5</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x22</td>
                    <td>s6</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x23</td>
                    <td>s7</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x24</td>
                    <td>s8</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x25</td>
                    <td>s9</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x26</td>
                    <td>s10</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x27</td>
                    <td>s11</td>
                    <td>0</td>
                </tr>
                <tr>
                    <td>x28</td>
                    <td>t3</td>
                    <td><input id="x28" type="number" value="0" /></td>
                </tr>
                <tr>
                    <td>x29</td>
                    <td>t4</td>
                    <td><input id="x29" type="number" value="0" /></td>
                </tr>
                <tr>
                    <td>x30</td>
                    <td>t5</td>
                    <td><input id="x30" type="number" value="0" /></td>
                </tr>
                <tr>
                    <td>x31</td>
                    <td>t6</td>
                    <td><input id="x31" type="number" value="0" /></td>
                </tr>
            </table>
        </div>
    
        <script>
            let line = -1;
            let programNumber = null;
    
            //Thanks https://webtips.dev/add-line-numbers-to-html-textarea !
            const textarea = document.querySelector('textarea')
            const lineNumbers = document.querySelector('.line-numbers')
    
            textarea.addEventListener('keyup', event => {
    
                const numberOfLines = event.target.value.split('\n').length
    
                lineNumbers.innerHTML = Array(numberOfLines)
                    .fill('<span class="line-number"></span>')
                    .join('')
            });
    
    
            function reset_code() {
                line = -1;
                programNumber = null;
                for (const element of document.getElementsByTagName('input')) {
                    element.readOnly = false
                }
                errorElem.style.visibility = "";
    
                const lines = document.getElementsByClassName('line-number');
                for (const line of lines) {
                    line.classList.remove('executed');
                }
            }
    
            const errorElem = document.getElementById("error");
            function step() {
                const code = document.querySelector('textarea');
    
                const registers = [];
    
                const table_rows = document.getElementsByTagName('tr');
    
                for (let i = 2; i < table_rows.length; i++) {
                    const thirdColumn = table_rows[i].children[2];
    
                    if (thirdColumn.textContent === "Can't read") {
                        registers.push(0);
                    } else if (thirdColumn.textContent) {
                        registers.push(thirdColumn.textContent); //x0 is zero, x1 is in index 0
                    } else {
                        registers.push(thirdColumn.children[0].value); // input elements
                    }
    
                }
    
                const body = programNumber ? '' : `[${registers}]\n${textarea.value}\n###\n`;
    
                const url = programNumber ? `/${programNumber}` : '/new'
    
                if (!programNumber) {
                    for (const element of document.getElementsByTagName('input')) {
                        element.readOnly = true
                    }
                }
    
                fetch(url, {
                    method: "POST",
                    body: body,
                    headers: {
                        "Accept": "application/json",
                    },
                }).then(response => {
                    if (!response.ok) {
                        return Promise.reject(response);
                    }
                    return response.json();
                }).then((data) => {
                    if (data.done) {
                        reset_code();
                        errorElem.style.visibility = "visible";
                        errorElem.children[0].textContent = "Program finished :)";
                        return;
                    }
                    programNumber = data.program_number;
                    errorElem.style.visibility = "";
                    if (data.error) {
                        errorElem.style.visibility = "visible";
                        errorElem.children[0].textContent = data.error;
                        return;
                    }
    
                    line = data.line; //executed line
    
                    const table_rows = document.getElementsByTagName('tr');
    
                    for (let i = 2; i < table_rows.length; i++) {
                        const thirdColumn = table_rows[i].children[2];
    
                        if (thirdColumn.textContent === "Can't read") {
                            continue;
                        } else if (thirdColumn.textContent) {
                            thirdColumn.textContent = data.registers[i - 2]; //x0 is zero, x1 is in index 0
                        } else {
                            thirdColumn.children[0].value = data.registers[i - 2]; // input elements
                        }
    
                    }
    
                    const lines = document.getElementsByClassName('line-number');
    
                    for (const line of lines) {
                        line.classList.remove('executed');
                    }
    
                    if (line > -1) {
                        lines[line-1].classList.add('executed');
                    }
                }).catch((err) => {
                    console.error(err);
                    errorElem.style.visibility = "visible";
                    errorElem.children[0].textContent = err.toString();
                });
    
            }
        </script>
    </body>
    
    </html>"#;

    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    response
}
