// Embassy - async Mutex

#![no_std]
#![no_main]

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex; // 使用 async 的 Mutex
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::timer::timg::TimerGroup;

esp_bootloader_esp_idf::esp_app_desc!();

static SHARED: Mutex<CriticalSectionRawMutex, u32> = Mutex::new(0);

#[embassy_executor::task]
async fn embassy_task() {
    loop {
        {
            let mut shared = SHARED.lock().await;
            *shared = shared.wrapping_add(1);
            Timer::after(Duration::from_millis(1000)).await;
        }
        Timer::after(Duration::from_millis(1000)).await;
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
        let shared = SHARED.lock().await;
        esp_println::println!("Current: {}", shared);
    }
}