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
        .with_sda(peripherals.GPIO3)
        .with_scl(peripherals.GPIO2);

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

    // 设置时间：逐个寄存器写入BCD值
    // 设置秒寄存器（同时激活振荡器，bit7=0启用）
    // BCD转换：BCD（Binary-Coded Decimal，二进制编码十进制）是将十进制数字用4位二进制表示的形式，DS1307使用BCD存储时间（如59秒为0x59）。nobcd crate用于十进制与BCD字节的转换，确保数据兼容。
    let secs: [u8; 1] = BcdNumber::new(start_dt.sec).unwrap().bcd_bytes();
    // I2C写入协议：发送起始位 + 从机地址 (写模式) + 数据切片（寄存器地址 + 值） + 停止位。从机返回ACK确认。
    // 如果失败（如无硬件响应NoAck），打印错误并继续（增强鲁棒性，避免panic）。
    if let Err(e) = ds1307.write(DS1307_ADDR, &[DS1307::Seconds as u8, secs[0]]) {
        println!("I2C write error (Seconds): {:?}", e); // 常见错误：NoAck（无应答）、Timeout等
    }

    // 设置分寄存器
    let mins: [u8; 1] = BcdNumber::new(start_dt.min).unwrap().bcd_bytes();
    if let Err(e) = ds1307.write(DS1307_ADDR, &[DS1307::Minutes as u8, mins[0]]) {
        println!("I2C write error (Minutes): {:?}", e);
    }

    // 设置时寄存器（假设24小时模式）
    let hrs: [u8; 1] = BcdNumber::new(start_dt.hrs).unwrap().bcd_bytes();
    if let Err(e) = ds1307.write(DS1307_ADDR, &[DS1307::Hours as u8, hrs[0]]) {
        println!("I2C write error (Hours): {:?}", e);
    }
    
    // 设置星期寄存器
    let dow: [u8; 1] = BcdNumber::new(start_dt.day).unwrap().bcd_bytes();
    if let Err(e) = ds1307.write(DS1307_ADDR, &[DS1307::Day as u8, dow[0]]) {
        println!("I2C write error (Day): {:?}", e);
    }
    
    // 设置日寄存器
    let dom: [u8; 1] = BcdNumber::new(start_dt.date).unwrap().bcd_bytes();
    if let Err(e) = ds1307.write(DS1307_ADDR, &[DS1307::Date as u8, dom[0]]) {
        println!("I2C write error (Date): {:?}", e);
    }
    
    // 设置月寄存器
    let mnth: [u8; 1] = BcdNumber::new(start_dt.mnth).unwrap().bcd_bytes();
    if let Err(e) = ds1307.write(DS1307_ADDR, &[DS1307::Month as u8, mnth[0]]) {
        println!("I2C write error (Month): {:?}", e);
    }
    
    // 设置年寄存器（0-99）
    let yr: [u8; 1] = BcdNumber::new(start_dt.yr).unwrap().bcd_bytes();
    if let Err(e) = ds1307.write(DS1307_ADDR, &[DS1307::Year as u8, yr[0]]) {
        println!("I2C write error (Year): {:?}", e);
    }
    
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

        // 解析BCD数据：秒（屏蔽bit7 CH位，CH=1表示振荡器停止）
        // 位掩码& 0x7f：去除bit7，只取0-59秒值。如果CH=1，时间无效，需重新设置。
        let secs = BcdNumber::from_bcd_bytes([data[0] & 0x7f])
            .unwrap()
            .value::<u8>();
        // 解析分
        let mins = BcdNumber::from_bcd_bytes([data[1]])
            .unwrap()
            .value::<u8>();
        // 解析时（屏蔽bit6-7，假设24小时模式）
        // 位掩码& 0x3f：去除模式位（bit6=0为24小时，=1为12小时）和AM/PM位（bit5）。代码假设24小时；如果12小时模式，需检查bit6并处理1-12 + AM/PM，否则解析错误。
        let hrs = BcdNumber::from_bcd_bytes([data[2] & 0x3f])
            .unwrap()
            .value::<u8>();
        // 潜在问题：若DS1307配置为12小时，未处理AM/PM，可能显示错误时间。建议添加bit6检查。
        // 解析日
        let dom = BcdNumber::from_bcd_bytes([data[4]])
            .unwrap()
            .value::<u8>();
        // 解析月
        let mnth = BcdNumber::from_bcd_bytes([data[5]])
            .unwrap()
            .value::<u8>();
        // 解析年
        let yr = BcdNumber::from_bcd_bytes([data[6]])
            .unwrap()
            .value::<u8>();
        // 解析星期，并映射为字符串
        let dow = match BcdNumber::from_bcd_bytes([data[3]])
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

        // 打印格式化时间（年份假设20xx）
        println!(
            "{}, {}/{}/20{}, {:02}:{:02}:{:02}",
            dow, dom, mnth, yr, hrs, mins, secs
        );

        // 延迟1秒
        delay.delay_millis(1000_u32);
    }
}