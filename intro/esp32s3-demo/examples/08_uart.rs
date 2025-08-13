/*
Simplified Embedded Rust: ESP Core Library Edition
Programming Serial Communication - Console Printing Application Example
*/
#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    uart::{Uart, Config},
    main,
};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // Configure Peripherals and System Clocks
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Create a Delay abstraction
    let delay = Delay::new();

    // Create a UART Configuration
    let uart_config = Config::default();

    let mut log = Uart::new(peripherals.UART0, uart_config)
        .unwrap()
    	.with_rx(peripherals.GPIO21)
    	.with_tx(peripherals.GPIO20);

    esp_println::print!("\x1b[20h");

    loop {
        println!("esp_println output");
        const MESSAGE: &[u8] = b"write method output \r\n";
        log.write(MESSAGE)
            .unwrap();
        delay.delay_millis(1000u32);
    }
}