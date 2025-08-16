// Embassy - Signal

#![no_std]
#![no_main]

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::rng::Rng;

esp_bootloader_esp_idf::esp_app_desc!();

static SHARED: Signal<CriticalSectionRawMutex, u32> = Signal::new();


#[embassy_executor::task]
async fn embassy_task(mut rng: Rng) {
    loop {
        let random = rng.random();
        SHARED.signal(random);
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
    let rng = Rng::new(peripherals.RNG);
    spawner.spawn(embassy_task(rng)).unwrap();

    loop {
        Timer::after(Duration::from_secs(1)).await;
        let shared = SHARED.wait().await;
        esp_println::println!("Current: {}", shared);
    }
}