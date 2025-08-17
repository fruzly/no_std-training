// Embassy Blinky

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;

use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    timer::timg::TimerGroup,
};

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // 获取外设 & 配置系统时钟
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    // 初始化 embassy 执行器，把硬件定时器交给 Embassy，作为其后台调度的时间基准
    // 告诉 Embassy 的运行时系统使用哪个硬件定时器（timg0.timer0）作为所有异步时间操作（如延迟、超时）的“心跳”时钟
    esp_hal_embassy::init(timg0.timer0);

    // 设置和配置 LED 输出引脚
    let mut led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());

    loop {
        // 翻转 LED 的状态
        led.toggle();
        Timer::after(Duration::from_millis(1000)).await;
    }
}