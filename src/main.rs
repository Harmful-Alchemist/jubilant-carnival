#![feature(let_chains)]

mod assembly;

use std::cell::RefCell;
use std::ops::BitOr;
use std::rc::Rc;
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

type Interpreters = Rc<RefCell<(u8,[Interpreter;5])>>;

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

    let interpreters: Interpreters = Rc::new(RefCell::new((0,[Interpreter::new(vec![]).unwrap(),Interpreter::new(vec![]).unwrap(),Interpreter::new(vec![]).unwrap(),Interpreter::new(vec![]).unwrap(),Interpreter::new(vec![]).unwrap()])));
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream, interpreters.clone());
    }

    fn handle_connection(mut stream: TcpStream, interpreters: Interpreters) {
        let buf_reader = BufReader::new(&mut stream);
        // let http_request: Vec<_> = buf_reader
        //     .lines()
        //     .map(|result| result.unwrap())
        //     .take_while(|line| !line.is_empty())
        //     .collect();
        // dbg!(&http_request);
        // let program = vec!["addi x31, x31, 1", "add x31, x31, x31"];
        // let mut interpreter = Interpreter::new(program).unwrap();

        // interpreter.step();

        // println!("After step 1:\n{:?}\n", interpreter.registers);

        // interpreter.step();

        // println!("After step 2:\n{:?}\n", interpreter.registers);

        let mut lines = buf_reader.lines();

        let line = lines.next().unwrap().unwrap();

        let response;
        if line.contains("GET") {
            response = get_resp();
        } else { //TODO new and the 
            //No error handling pretend everything else is a POST request TODO do some error handling so doesn't crash
            // let content_length = "Content-Length: ";
            // let length = http_request
            //     .iter()
            //     .filter(|h| h.starts_with(content_length))
            //     .map(|h| h.replace(content_length, ""))
            //     .map(|h| h.parse::<u64>())
            //     .next()
            //     .unwrap()
            //     .unwrap();

            // let buf_reader = BufReader::new(&mut stream);
            // let bodyish: Vec<_> = buf_reader.lines().map(|result| result.unwrap()).take(3).collect();

            // let mut string_buf = String::new();
            // buf_reader.take(length).read_to_string(&mut string_buf);

            // dbg!(&bodyish);
            
            let status_line = "HTTP/1.1 200 OK";
            let mut contents = String::new();

            loop {
                match lines.next() {
                    Some(line) => {
                        dbg!(&line);
                        let line = match line {
                            Ok(l) => l,
                            Err(e) => {dbg!(e); break},
                        };
                        dbg!(&line);
                        contents.push_str(&line);
                        if line.contains("###") {
                            break;
                        }}
                    None => {dbg!("No more lines");break},
                }
            }

            let length = contents.len();
            response = format!("{status_line}\r\nContent-Length: {length}\r\nContent-Type: application/json\r\n\r\n{contents}");
        }
        stream.write_all(response.as_bytes()).unwrap();
    }
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
add x31, x31, 10
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
            document.getElementsByTagName('input').array.forEach(element => {
                    element.readOnly = false
                });
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

            const body = `${registers}\n${line + 1}\n${textarea.value}\n###\n`;

            const url = programNumber ? `/${programNumber}` : '/new'

            if (programNumber) {
                document.getElementsByTagName('input').array.forEach(element => {
                    element.readOnly = true
                });
            }

            fetch(url, {
                method: "POST",
                body: body,
                headers: {
                    "Accept": "application/json",
                },
            })
                .then((data) => {

                    const resp = data.json();
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

                    for (const line in lines) {
                        if (Object.hasOwnProperty.call(object, line)) {
                            line.classList.remove('executed');
                            
                        }
                    }

                    if (line > -1) {
                        lines[line].classList.add('executed');
                    }
                });

        }
    </script>
</body>

</html>"#;

    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    response
}
