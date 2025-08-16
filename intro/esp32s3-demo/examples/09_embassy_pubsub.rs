// Embassy - PubSub

#![no_std]
#![no_main]

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::PubSubChannel;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::timer::timg::TimerGroup;

esp_bootloader_esp_idf::esp_app_desc!();

// 声明一个容量为2、订阅者为2、发布者为2 的PubSubChannel，用于发送和接收数据
static SHARED: PubSubChannel<CriticalSectionRawMutex, u32, 2, 2, 2> = PubSubChannel::new();


#[embassy_executor::task]
async fn embassy_task_one() {
    let publisher = SHARED.publisher().unwrap();
    loop {
        publisher.publish_immediate(1);
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn embassy_task_two() {
    let publisher = SHARED.publisher().unwrap();
    loop {
        publisher.publish_immediate(2);
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

    // 创建2个发布任务
    spawner.spawn(embassy_task_one()).unwrap();
    spawner.spawn(embassy_task_two()).unwrap();

    // 创建一个订阅任务打印接收到的数据
    let mut subscriber = SHARED.subscriber().unwrap();
    loop {
        let shared = subscriber.next_message_pure().await;
        esp_println::println!("Current: {}", shared);
    }
}