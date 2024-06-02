#![feature(let_chains)]

mod assembly;

use assembly::Interpreter;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // let instruction = 0;
    // let mut zeroes = [1; 31];
    // zeroes[4] = 10;
    // let registers = assembly::step(instruction, zeroes);

    // log::info!("Hello, world! , {registers:?}");
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
}
