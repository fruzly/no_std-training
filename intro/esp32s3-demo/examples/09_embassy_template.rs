// Embassy 模板
// 注意：Cargo.toml 中 embassy-executor 的版本需要与 esp-hal-embassy 依赖的 embassy-executor 的版本一致

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::timer::timg::TimerGroup;

esp_bootloader_esp_idf::esp_app_desc!();

#[embassy_executor::task]
async fn embassy_task() {
    esp_println::println!("Embassy task started!");

    loop {
        esp_println::println!("Embassy task loop!");
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::println!("Init!");

    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    // 初始化 embassy executor
    esp_hal_embassy::init(timg0.timer0);

    // 创建一个任务
    spawner.spawn(embassy_task()).unwrap();

    // 等待任务完成
    loop {
        esp_println::println!("Main loop!");
        Timer::after(Duration::from_secs(2)).await;
    }
}