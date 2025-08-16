// 修改文件标题和描述
// 简化嵌入式Rust: ESP核心库版
// 编程串行通信 - I2C MPU-6050 传感器示例
// 此示例演示如何使用ESP32-S3的I2C接口与MPU-6050通信，初始化传感器并循环读取加速度和陀螺仪数据。

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay, i2c::master::{Config, I2c}, main, time::Rate,
};
use esp_println::println;

// MPU-6050的I2C从机地址（AD0接地为0x68）
const MPU6050_ADDR: u8 = 0x68;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // 初始化ESP外设，使用默认配置
    let peripherals = esp_hal::init(esp_hal::Config::default());
    // 创建延迟对象，用于时间延迟
    let delay = Delay::new();

    // 创建I2C配置，默认配置并设置频率为100kHz（DS1307标准速率）
    let i2c_config = Config::default().with_frequency(Rate::from_khz(100));

    // 初始化I2C主设备，使用I2C0硬件，指定SDA (GPIO3) 和 SCL (GPIO2) 引脚
    let mut ds1307 = I2c::new(peripherals.I2C0, i2c_config)
        .unwrap()
        .with_sda(peripherals.GPIO4)
        .with_scl(peripherals.GPIO5);
        // .with_sda(peripherals.GPIO3)
        // .with_scl(peripherals.GPIO2);

    // 初始化 MPU-6050：写寄存器 0x6B (PWR_MGMT_1) 为 0x00 以唤醒
    if let Err(e) = ds1307.write(MPU6050_ADDR, &[0x6B, 0x00]) {
        println!("Failed to wake MPU-6050: {:?}", e);
    } else {
        println!("MPU-6050 initialized.");
    }

    // 主循环：每秒读取并打印时间
    loop {
        // 读取 14 字节数据：加速度 (6字节) + 温度 (2字节) + 陀螺仪 (6字节)，从 0x3B 开始
        let mut data: [u8; 14] = [0; 14];
        if let Err(e) = ds1307.write(MPU6050_ADDR, &[0x3B]) {
            println!("I2C write error (Address): {:?}", e);
            delay.delay_millis(1000_u32);
            continue;
        }
        if let Err(e) = ds1307.read(MPU6050_ADDR, &mut data) {
            println!("I2C read error: {:?}", e);
            delay.delay_millis(1000_u32);
            continue;
        }

        // 解析数据（16-bit 有符号整数，高字节先）
        let accel_x = (data[0] as i16) << 8 | data[1] as i16;
        let accel_y = (data[2] as i16) << 8 | data[3] as i16;
        let accel_z = (data[4] as i16) << 8 | data[5] as i16;
        let gyro_x = (data[8] as i16) << 8 | data[9] as i16;
        let gyro_y = (data[10] as i16) << 8 | data[11] as i16;
        let gyro_z = (data[12] as i16) << 8 | data[13] as i16;

        // 打印
        println!(
            "Accel: X={:5} Y={:5} Z={:5} | Gyro: X={:5} Y={:5} Z={:5}",
            accel_x, accel_y, accel_z, gyro_x, gyro_y, gyro_z
        );

        delay.delay_millis(1000_u32);
    }
}