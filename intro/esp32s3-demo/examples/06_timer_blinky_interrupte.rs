// 定时器中断示例 - 使用 ISR 周期触发 LED 闪烁

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    handler, main,
    timer::timg::TimerGroup,
    time::Duration,
};
use esp_println::println;
use core::{
    cell::RefCell,
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
};
use critical_section::Mutex;
// 关键：引入 Timer trait 以启用其方法（如 load_value/enable_interrupt/clear_interrupt），解决私有方法报错
use esp_hal::timer::Timer as _;

esp_bootloader_esp_idf::esp_app_desc!();

// 在 ISR 与主循环之间共享定时器实例与事件标志
static G_TIMER: Mutex<RefCell<Option<esp_hal::timer::timg::Timer>>> =
    Mutex::new(RefCell::new(None));
static TICK: AtomicBool = AtomicBool::new(false);
// 用于演示发布-获取：
// - 当启用 feature `publish_before` 时：ISR 在 Release 发布前先写 COUNTER
// - 默认情况下（未启用）：ISR 在 Release 发布后再写 COUNTER
// 主循环使用 Acquire 获取后读取 COUNTER，用于观察可见性差异
static COUNTER: AtomicU32 = AtomicU32::new(0);

#[handler]
fn timer_isr() {
    // 清除中断标志，并设置一次性事件
    critical_section::with(|cs| {
        if let Some(t) = G_TIMER.borrow_ref_mut(cs).as_mut() {
            t.clear_interrupt();
        }
    });
    // 根据 feature 切换演示顺序：
    #[cfg(feature = "publish_before")]
    {
        // 发布前写入：这次写入将被随后 TICK 的 Release 所发布
        // 发布前写入，主循环通过 Acquire 能看到增量
        let _ = COUNTER.fetch_add(1, Ordering::Relaxed);
        TICK.store(true, Ordering::Release);
    }
    #[cfg(not(feature = "publish_before"))]
    {
        // 发布后写入：这次写入不会被上一次 Release/Acquire 所保证可见
        TICK.store(true, Ordering::Release); // 原为 Relaxed：仅事件标志可用 Relaxed；此处改为 Release
        // 发布后写入，Acquire 不保证可见
        let _ = COUNTER.fetch_add(1, Ordering::Relaxed);
    }
}

// 默认模式：cargo run --example 06_timer_blinky_interrupte
// 在 publish_before 模式下：cargo run --example 06_timer_blinky_interrupte --features publish_before

#[main]
fn main() -> ! {
    // 获取外设工具箱
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // 配置 LED：将 gpio4 设置为输出引脚，初始状态为低电平
    let mut led = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());

    // 创建定时器组与定时器
    let timer_group = TimerGroup::new(peripherals.TIMG0);
    let timer0 = timer_group.timer0;

    // 配置周期 500ms，并启用自动重载 + 中断 + ISR
    timer0.enable_auto_reload(true);
    timer0.load_value(Duration::from_millis(500)).unwrap();
    timer0.set_interrupt_handler(timer_isr);
    timer0.enable_interrupt(true);
    timer0.start();

    // 将定时器移入全局，供 ISR 清除中断标志使用
    critical_section::with(|cs| G_TIMER.borrow_ref_mut(cs).replace(timer0));

    println!("Timer ISR example started! LED toggles every 500ms");

    // 主循环：响应 ISR 设置的事件
    let mut count: u32 = 0;
    loop {
        // Acquire：与 ISR 的 Release 配合，用于演示发布-获取
        if TICK.swap(false, Ordering::Acquire) {
            // 读取 ISR 在 Release 之后写入的数据：不保证可见
            let before = COUNTER.load(Ordering::Relaxed);
            // 在 main 中对该数据进行修改（演示写入）
            let after = COUNTER.fetch_add(10, Ordering::Relaxed) + 10;
            led.toggle();
            count += 1;
            println!("tick={} counter_before={} counter_after={}", count, before, after);
        }
    }
}