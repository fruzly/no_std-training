// Embassy - Channel

#![no_std]
#![no_main]

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::timer::timg::TimerGroup;

esp_bootloader_esp_idf::esp_app_desc!();

// 创建一个容量为2的通道，用于发送和接收数据
static SHARED: Channel<CriticalSectionRawMutex, u32, 2> = Channel::new();

#[embassy_executor::task]
async fn embassy_task_sender1() {
    loop {
        SHARED.send(1).await;
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn embassy_task_sender2() {
    loop {
        SHARED.send(2).await;
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // 初始化并创建设备外设的句柄
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    // 初始化 embassy 执行器
    esp_hal_embassy::init(timg0.timer0);
    // 创建2个任务
    spawner.spawn(embassy_task_sender1()).unwrap();
    spawner.spawn(embassy_task_sender2()).unwrap();

    loop {
        Timer::after(Duration::from_millis(500)).await;
        let shared = SHARED.receive().await;
        esp_println::println!("Current: {}", shared);
    }
}