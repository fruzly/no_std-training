// 中断

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    gpio::{Event, Input, InputConfig, Io, Level, Output, OutputConfig, Pull},
    handler, main,
};
use core::sync::atomic::{AtomicU32, Ordering};
use core::cell::{Cell, RefCell};
use critical_section::Mutex;
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

// 用于在 ISR 和 main 之间共享引脚实例，以便在 ISR 中清除中断标志
static G_PIN: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));
// 用于在 ISR 和 main 之间传递“按钮被按下”这一事件的标志位
static G_FLAG: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
// 全局变量：用于存储闪烁延时，AtomicU32 是线程安全的整数类型
static BLINK_DELAY: AtomicU32 = AtomicU32::new(500); // 初始延时 500ms

#[handler] // 这是一个中断处理函数
fn gpio_handler() {
    // 进入临界区，只做必要的硬件操作
    critical_section::with(|cs| {
        // 清除中断标志 - 这是硬件必需的操作
        G_PIN.borrow_ref_mut(cs).as_mut().unwrap().clear_interrupt();
        // 设置标志位，通知 main 循环
        G_FLAG.borrow(cs).set(true);
    });
    // ISR 结束 - 总执行时间应该在微秒级别
}

#[main]
fn main() -> ! {
    // 获取外设工具箱
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // 创建 GPIO 总管家
    let mut io = Io::new(peripherals.IO_MUX);
    // 1. 注册中断处理函数
    io.set_interrupt_handler(gpio_handler);
    println!("GPIO interrupt handler registered");

    // 配置 LED：将 gpio4 设置为输出引脚，初始状态为高电平
    let mut led = Output::new(peripherals.GPIO4, Level::High, OutputConfig::default());

    let config = InputConfig::default().with_pull(Pull::Up);

    // 2. 配置引脚为输入
    // 配置按钮：将 gpio9 设置为输入引脚，并启用内部上拉电阻
    let mut button = Input::new(peripherals.GPIO9, config);
    println!("button level {:?} - {:?} - {:#?}", button.is_low(), button.is_high(), button.level());
    // 3. 监听下降沿事件
    button.listen(Event::FallingEdge);

    // 4. 将配置好的引脚移入全局变量
    critical_section::with(|cs| G_PIN.borrow_ref_mut(cs).replace(button));

    let mut delay = esp_hal::delay::Delay::new();

    // 进入主循环
    loop {
        // 延时循环结束后（也就是过了一段时间后），翻转 LED 的状态
        led.toggle();

        // 获取当前的延时值
        let current_delay = BLINK_DELAY.load(Ordering::Relaxed);
        // 硬件延时
        delay.delay_millis(current_delay);
        
        critical_section::with(|cs| {
            if G_FLAG.borrow(cs).get() {
                // Clear global flag
                G_FLAG.borrow(cs).set(false);

                // 修改延时值
                let mut new_delay = BLINK_DELAY.load(Ordering::Relaxed);
                new_delay = if new_delay <= 100 { 500 } else { new_delay - 100 };
                BLINK_DELAY.store(new_delay, Ordering::Relaxed);
                esp_println::println!("Delay changed to: {}", new_delay);
            }
        });
    }
}