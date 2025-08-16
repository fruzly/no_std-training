// Embassy - Mutex

#![no_std]
#![no_main]

use core::cell::RefCell;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::timer::timg::TimerGroup;

esp_bootloader_esp_idf::esp_app_desc!();

static SHARED: Mutex<CriticalSectionRawMutex, RefCell<u32>> = Mutex::new(RefCell::new(0));


#[embassy_executor::task]
async fn embassy_task() {
    loop {
        // 从全局上下文加载值，修改并存储
        SHARED.lock(|shared| {
            // *shared.borrow_mut() += 1;
            let val = shared.borrow_mut().wrapping_add(1);
            shared.replace(val);
        });
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
        let shared = SHARED.lock(|shared| {
            shared.clone().into_inner()
        });
        Timer::after(Duration::from_secs(1)).await;
        esp_println::println!("Current: {}", shared);
    }
}