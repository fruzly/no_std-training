// 简化嵌入式Rust: ESP核心库版
// 编程串行通信 - I2C实时钟应用示例
// 此示例演示如何使用ESP32-S3的I2C接口与DS1307 RTC芯片通信，设置初始时间并循环读取打印。
// MPU6050 I2C 示例 (基于 esp-hal)

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{delay::Delay, i2c::master::{Config, I2c}, main, time::Rate};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

const MPU6050_ADDR: u8 = 0x69; // AD0 高电平

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    let i2c_config = Config::default().with_frequency(Rate::from_khz(400)); // MPU6050 支持更高频率
    let mut i2c = I2c::new(peripherals.I2C0, i2c_config)
        .unwrap()
        .with_sda(peripherals.GPIO4)
        .with_scl(peripherals.GPIO5);

    // 唤醒 MPU6050 (寄存器 0x6B, 值 0x00)
    if let Err(e) = i2c.write(MPU6050_ADDR, &[0x6B, 0x00]) {
        println!("I2C write error: {:?}", e);
    }

    loop {
        let mut data: [u8; 6] = [0; 6]; // 读取加速度 (寄存器 0x3B-0x40)
        if let Err(e) = i2c.write(MPU6050_ADDR, &[0x3B]) {
            println!("I2C write error: {:?}", e);
            delay.delay_millis(1000);
            continue;
        }
        if let Err(e) = i2c.read(MPU6050_ADDR, &mut data) {
            println!("I2C read error: {:?}", e);
            delay.delay_millis(1000);
            continue;
        }

        // 解析数据 (16位有符号整数)
        let accel_x = ((data[0] as i16) << 8) | data[1] as i16;
        let accel_y = ((data[2] as i16) << 8) | data[3] as i16;
        let accel_z = ((data[4] as i16) << 8) | data[5] as i16;

        println!("Accel: X={}, Y={}, Z={}", accel_x, accel_y, accel_z);
        delay.delay_millis(1000);
    }
}