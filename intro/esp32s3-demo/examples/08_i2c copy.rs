// 简化嵌入式Rust: ESP核心库版
// 编程串行通信 - I2C实时钟应用示例
// 此示例演示如何使用ESP32-S3的I2C接口与DS1307 RTC芯片通信，设置初始时间并循环读取打印。

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay, i2c::master::{Config, I2c}, main, time::Rate,
};
use esp_println::println;
use nobcd::BcdNumber;

// DS1307的I2C从机地址，固定为0x68（7位地址）
const DS1307_ADDR: u8 = 0x68; 

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

    // 定义DS1307寄存器地址枚举（u8表示），对应手册中寄存器偏移
    #[repr(u8)]
    enum DS1307 {
        Seconds,  // 0x00: 秒寄存器
        Minutes,  // 0x01: 分寄存器
        Hours,    // 0x02: 时寄存器
        Day,      // 0x03: 星期寄存器
        Date,     // 0x04: 日寄存器
        Month,    // 0x05: 月寄存器
        Year,     // 0x06: 年寄存器
    }

    // 定义星期枚举，DS1307使用1-7表示周日到周六
    enum DAY {
        Sun = 1,
        Mon = 2,
        Tues = 3,
        Wed = 4,
        Thurs = 5,
        Fri = 6,
    }

    // 定义时间结构体，用于存储初始时间数据
    struct DateTime {
        sec: u8,   // 秒 (0-59)
        min: u8,   // 分 (0-59)
        hrs: u8,   // 时 (0-23, 假设24小时模式)
        day: u8,   // 星期 (1-7)
        date: u8,  // 日 (1-31)
        mnth: u8,  // 月 (1-12)
        yr: u8,    // 年 (0-99, 如15表示2015)
    }

    // 创建初始时间实例（示例: 2015-05-15 周五 00:00:00）
    let start_dt = DateTime {
        sec: 0,
        min: 0,
        hrs: 0,
        day: DAY::Fri as u8,
        date: 15,
        mnth: 5,
        yr: 15,
    };

    // 修改为批量写入所有时间寄存器（从0x00开始）
    let sec_bcd = BcdNumber::<1>::new(start_dt.sec).unwrap().bcd_bytes()[0] & 0x7f; // 确保 CH=0
    let min_bcd = BcdNumber::<1>::new(start_dt.min).unwrap().bcd_bytes()[0];
    let hr_bcd = BcdNumber::<1>::new(start_dt.hrs).unwrap().bcd_bytes()[0]; // 假设24h, bit6=0
    let day_bcd = BcdNumber::<1>::new(start_dt.day).unwrap().bcd_bytes()[0];
    let date_bcd = BcdNumber::<1>::new(start_dt.date).unwrap().bcd_bytes()[0];
    let mnth_bcd = BcdNumber::<1>::new(start_dt.mnth).unwrap().bcd_bytes()[0];
    let yr_bcd = BcdNumber::<1>::new(start_dt.yr).unwrap().bcd_bytes()[0];

    let write_buf = [0x00, sec_bcd, min_bcd, hr_bcd, day_bcd, date_bcd, mnth_bcd, yr_bcd];
    if let Err(e) = ds1307.write(DS1307_ADDR, &write_buf) {
        println!("I2C batch write error: {:?}", e);
    } else {
        println!("Time set successfully.");
    }

    // 立即读取验证
    let mut verify_data: [u8; 7] = [0; 7];
    if ds1307.write(DS1307_ADDR, &[0x00]).is_ok() {
        if ds1307.read(DS1307_ADDR, &mut verify_data).is_ok() {
            println!("Verify raw data after set: {:?}", verify_data);
        }
    }

    // 添加诊断: 延迟2秒后再次读取，检查秒是否递增
    println!("Waiting 2 seconds to check if clock is running...");
    delay.delay_millis(2000_u32);
    let mut check_data: [u8; 7] = [0; 7];
    match ds1307.write(DS1307_ADDR, &[0x00]) {
        Ok(_) => {
            match ds1307.read(DS1307_ADDR, &mut check_data) {
                Ok(_) => {
                    println!("Data after 2s: {:?}", check_data);
                    if check_data[0] == verify_data[0] {
                        println!("Warning: Clock not advancing! Check hardware (crystal/battery).");
                    } else {
                        println!("Clock is running.");
                    }
                }
                Err(e) => {
                    println!("I2C read error: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("I2C write error: {:?}", e);
        }
    };
    

    // 主循环：每秒读取并打印时间
    loop {
        // 初始化7字节缓冲区，用于存储从DS1307读取的数据（寄存器0x00-0x06）
        let mut data: [u8; 7] = [0_u8; 7];

        // 写入起始寄存器地址0x00，准备连续读取
        // I2C读取协议：先写起始地址（设置指针），然后读数据。涉及重启起始位以切换读模式。
        if let Err(e) = ds1307.write(DS1307_ADDR, &[0_u8]) {
            println!("I2C write error (Address): {:?}", e); // 错误处理：打印并延迟1秒继续循环
            delay.delay_millis(1000_u32);
            continue;
        }
        // 读取7字节数据
        if let Err(e) = ds1307.read(DS1307_ADDR, &mut data) {
            println!("I2C read error: {:?}", e); // 读取失败时继续，避免程序崩溃
            delay.delay_millis(1000_u32);
            continue;
        }

        // 调试: 打印 raw data
        println!("Raw data: {:?}", data);

        // 检查 CH 位，如果为1则重置秒寄存器启用时钟
        if data[0] & 0x80 != 0 {
            println!("Clock halted, resetting...");
            let secs_reset: [u8; 1] = BcdNumber::<1>::new(0).unwrap().bcd_bytes();
            if let Err(e) = ds1307.write(DS1307_ADDR, &[DS1307::Seconds as u8, secs_reset[0]]) {
                println!("Failed to reset clock: {:?}", e);
            }
        }

        // 解析BCD数据：秒（屏蔽bit7 CH位，CH=1表示振荡器停止）
        // 位掩码& 0x7f：去除bit7，只取0-59秒值。如果CH=1，时间无效，需重新设置。
        let secs = BcdNumber::<1>::from_bcd_bytes([data[0] & 0x7f])
            .unwrap()
            .value::<u8>();
        // 解析分
        let mins = BcdNumber::<1>::from_bcd_bytes([data[1]])
            .unwrap()
            .value::<u8>();
        // 修改小时解析以支持 12/24 模式
        let hrs_reg = data[2];
        let hrs: u8;
        let mut is_pm = false; // 默认
        if hrs_reg & 0x40 != 0 { // bit6=1: 12小时模式
            hrs = BcdNumber::<1>::from_bcd_bytes([hrs_reg & 0x1f]).unwrap().value::<u8>();
            is_pm = (hrs_reg & 0x20) != 0; // bit5=1: PM
        } else { // 24小时模式
            hrs = BcdNumber::<1>::from_bcd_bytes([hrs_reg & 0x3f]).unwrap().value::<u8>();
        }
        // 潜在问题：若DS1307配置为12小时，未处理AM/PM，可能显示错误时间。建议添加bit6检查。
        // 解析日
        let dom = BcdNumber::<1>::from_bcd_bytes([data[4]])
            .unwrap()
            .value::<u8>();
        // 解析月
        let mnth = BcdNumber::<1>::from_bcd_bytes([data[5]])
            .unwrap()
            .value::<u8>();
        // 解析年
        let yr = BcdNumber::<1>::from_bcd_bytes([data[6]])
            .unwrap()
            .value::<u8>();
        // 解析星期，并映射为字符串
        let dow = match BcdNumber::<1>::from_bcd_bytes([data[3]])
            .unwrap()
            .value::<u8>()
        {
            1 => "Sunday",
            2 => "Monday",
            3 => "Tuesday",
            4 => "Wednesday",
            5 => "Thursday",
            6 => "Friday",
            7 => "Saturday",
            _ => "",  // 无效值处理
        };

        // 修改打印，添加 PM/AM 并修正年份
        let full_year = 2000 + yr as u32;
        let am_pm = if is_pm { " PM" } else { " AM" };
        println!(
            "{}, {}/{}/{}, {:02}:{:02}:{:02}{}",
            dow, dom, mnth, full_year, hrs, mins, secs, if hrs_reg & 0x40 != 0 { am_pm } else { "" }
        );

        // 延迟1秒
        delay.delay_millis(1000_u32);
    }
}