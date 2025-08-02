// blinky 发光二极管：包括 ESP32-S3 开发板上自带的LED 和接 GPIO7 的 RGB LED
/*
Simplified Embedded Rust: ESP Core Library Edition
Programming GPIO - Blinky Application Example
*/

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{
        DriveMode, DriveStrength, Level, Output,
        OutputConfig, Pull,
    },
    main,
    timer::timg::TimerGroup,
};
use esp_println::println;
use smart_leds::RGB8;

esp_bootloader_esp_idf::esp_app_desc!();

// 使用WS2812协议关闭开发板上的彩色 RGB LED (GPIO48)
// 简化的WS2812控制函数
fn send_ws2812_reset(pin: &mut Output, delay: &mut Delay) {
    // 发送复位信号：保持低电平超过50us
    pin.set_low();
    delay.delay_micros(100); // 100us复位信号
}

fn send_ws2812_bit(pin: &mut Output, bit: bool) {
    if bit {
        // 发送1: 高电平0.8us, 低电平0.4us
        pin.set_high();
        // 调整时序 - 更保守的延迟
        for _ in 0..60 { } // ~0.8us
        pin.set_low();
        for _ in 0..30 { } // ~0.4us
    } else {
        // 发送0: 高电平0.4us, 低电平0.8us  
        pin.set_high();
        for _ in 0..30 { } // ~0.4us
        pin.set_low();
        for _ in 0..60 { } // ~0.8us
    }
}

fn send_ws2812_byte(pin: &mut Output, byte: u8) {
    for i in (0..8).rev() {
        let bit = (byte >> i) & 1 == 1;
        send_ws2812_bit(pin, bit);
    }
}

fn send_ws2812_color(pin: &mut Output, color: RGB8) {
    // WS2812使用GRB顺序
    send_ws2812_byte(pin, color.g);
    send_ws2812_byte(pin, color.r);
    send_ws2812_byte(pin, color.b);
}

#[main]
fn main() -> ! {
    println!("\n=== ESP32-S3 Blinky Application ===");
    
    // Take the peripherals
    let peripherals =
        esp_hal::init(esp_hal::Config::default());

    // 禁用看门狗定时器
    let timer_group0 = TimerGroup::new(peripherals.TIMG0);
    let mut wdt0 = timer_group0.wdt;
    wdt0.disable();
    
    let timer_group1 = TimerGroup::new(peripherals.TIMG1);
    let mut wdt1 = timer_group1.wdt;
    wdt1.disable();
    
    println!("Watchdog timers disabled!");

    // Create a delay handle
    let mut delay = Delay::new();

    // Create output pin configuration
    let led_pin_conf = OutputConfig::default()
        .with_drive_mode(DriveMode::PushPull)
        .with_drive_strength(DriveStrength::_10mA)
        .with_pull(Pull::None);

    // Create output pin for external LED (GPIO7)
    let mut led_pin = Output::new(
        peripherals.GPIO7,
        Level::Low,
        led_pin_conf,
    );

    // 使用WS2812协议关闭开发板上的RGB LED (GPIO48)
    let rgb_led_conf = OutputConfig::default()
        .with_drive_mode(DriveMode::PushPull)
        .with_drive_strength(DriveStrength::_20mA)
        .with_pull(Pull::None);
    
    let mut rgb_led_pin = Output::new(
        peripherals.GPIO48,
        Level::Low,
        rgb_led_conf,
    );
    
    println!("Sending WS2812 black color to turn off RGB LED...");
    
    // 先发送复位信号
    send_ws2812_reset(&mut rgb_led_pin, &mut delay);
    
    // 发送多个黑色(0,0,0)像素，确保所有可能的LED都被关闭
    // let black_color = RGB8::new(0, 0, 0);
    // for _ in 0..8 { // 发送8个黑色像素
    //     send_ws2812_color(&mut rgb_led_pin, black_color);
    // }
    
    // 最后再发送复位信号
    // send_ws2812_reset(&mut rgb_led_pin, &mut delay);
    
    println!("WS2812 RGB LED should now be OFF (sent 8 black pixels)");
    println!("Starting LED blink loop on GPIO7...");
    let mut counter = 0;
    
    loop {
        // Turn on LED
        led_pin.set_high();
        println!("[{}] LED ON", counter);
        // Wait for 1 second
        delay.delay_millis(1000u32);
        
        // Turn off LED
        led_pin.set_low();
        println!("[{}] LED OFF", counter);
        // Wait for 1 second
        delay.delay_millis(1000u32);
        
        counter += 1;
    }
}