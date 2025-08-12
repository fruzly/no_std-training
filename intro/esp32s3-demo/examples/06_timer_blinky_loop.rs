// 定时器 - 循环 - LED 闪烁

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    timer::{timg::TimerGroup, Timer},
    main,
};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // 获取外设工具箱
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // 创建定时器组与定时器
    let timer_group0 = TimerGroup::new(peripherals.TIMG0);
    let timer0 = timer_group0.timer0;

    // 配置 LED：将 gpio4 设置为输出引脚，初始状态为高电平
    let mut led = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());

    let mut start = timer0.now();

    timer0.start();

    // 进入主循环
    loop {
        if start.elapsed().as_secs() > 1 {
            // 延时1秒后，翻转 LED 的状态
            led.toggle();
            start = timer0.now();
        }
    }
}
