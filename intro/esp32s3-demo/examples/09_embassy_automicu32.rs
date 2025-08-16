// Embassy - AutomicU32

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::timer::timg::TimerGroup;

esp_bootloader_esp_idf::esp_app_desc!();

static SHARED: AtomicU32 = AtomicU32::new(0);


#[embassy_executor::task]
async fn embassy_task() {
    loop {
        // 从全局上下文加载值，修改并存储
        let current = SHARED.load(Ordering::Relaxed);
        SHARED.store(current.wrapping_add(1), Ordering::Relaxed);
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // 初始化并创建设备外设的句柄
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    // 初始化 embassy 执行器
    esp_hal_embassy::init(timg0.timer0);
    // 创建一个任务
    spawner.spawn(embassy_task()).unwrap();

    loop {
        Timer::after(Duration::from_secs(1)).await;
        let shared = SHARED.load(Ordering::Relaxed);
        esp_println::println!("Current: {}", shared);
    }
}