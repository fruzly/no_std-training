#![no_std]
#![no_main]

// 导入核心库、临界区、Mutex 和原子类型
use core::cell::RefCell;
use critical_section::Mutex;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
// 导入 esp-hal 相关库
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Event, Input, InputConfig, Io, Pull},
    handler, main,
};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

// 1. 定义一个全局、线程安全的变量来持有 GPIO 引脚
// Mutex: 保证线程安全
// RefCell: 提供内部可变性，让我们能在 ISR 和 main 中都能修改它
// Option: 允许在 main 中初始化它
static G_PIN: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));

// 2. 定义原子变量用于 ISR 和主循环之间的通信
// 中断触发标志：当发生中断时设置为 true
static INTERRUPT_FLAG: AtomicBool = AtomicBool::new(false);
// 中断计数器：记录中断发生的次数
static INTERRUPT_COUNT: AtomicU32 = AtomicU32::new(0);
// GPIO状态：记录最后一次中断时的GPIO状态 (true=HIGH, false=LOW)
static LAST_GPIO_STATE: AtomicBool = AtomicBool::new(true);

// 3. 定义轻量化的中断服务例程 (ISR)
// ISR 原则：快速进入，快速退出，不做阻塞性操作
#[handler] // 这是一个中断处理函数
fn gpio_handler() {
    // 进入临界区，只做必要的硬件操作
    critical_section::with(|cs| {
        // 读取当前GPIO状态并保存到原子变量中
        let is_high = G_PIN.borrow_ref(cs).as_ref().unwrap().is_high();
        LAST_GPIO_STATE.store(is_high, Ordering::Relaxed);
        
        // 增加中断计数器
        INTERRUPT_COUNT.fetch_add(1, Ordering::Relaxed);
        
        // 设置中断标志，通知主循环处理
        INTERRUPT_FLAG.store(true, Ordering::Relaxed);

        // 清除中断标志 - 这是硬件必需的操作
        G_PIN.borrow_ref_mut(cs).as_mut().unwrap().clear_interrupt();
    });
    // ISR 结束 - 总执行时间应该在微秒级别
}

#[main]
fn main() -> ! {
    // --- 配置阶段 ---
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut io = Io::new(peripherals.IO_MUX);

    // --- 中断配置三部曲 ---
    println!("=== Starting GPIO interrupt configuration ===");
    // 第1步: 注册 ISR。告诉硬件，当 GPIO 中断发生时，调用名为 `gpio_handler` 的函数
    io.set_interrupt_handler(gpio_handler);
    println!("✓ Step 1: GPIO interrupt handler registered");

    let config = InputConfig::default().with_pull(Pull::Up);
    // 第2步: 配置引脚。将 gpio0 配置为带上拉电阻的输入引脚
    let mut some_pin = Input::new(peripherals.GPIO4, config);
    println!("✓ Step 2: GPIO4 configured as input with pull-up");

    // 第3步: 配置事件并开始监听。告诉硬件我们关心的是“下降沿”事件（电平从高到低的变化）
    some_pin.listen(Event::FallingEdge);
    println!("✓ Step 3: GPIO4 listening for FallingEdge events");

    // 检查初始引脚状态
    println!("Initial GPIO4 state: {}", if some_pin.is_high() { "HIGH" } else { "LOW" });

    // --- 将配置好的引脚移入全局变量 ---
    // 进入临界区，安全地将 `some_pin` 的所有权转移到 G_PIN 中
    critical_section::with(|cs| G_PIN.borrow_ref_mut(cs).replace(some_pin));
    println!("✓ GPIO4 pin moved to global variable");
    println!("=== GPIO interrupt configuration complete ===");

    let delay = Delay::new();
    let mut counter = 0;
    // --- 逻辑实现阶段 ---
    // 程序在这里进入一个空的无限循环，CPU 可以“休息”
    // 所有工作都将由中断触发
    loop {
        // 检查是否有中断发生
        if INTERRUPT_FLAG.load(Ordering::Relaxed) {
            // 重置中断标志
            INTERRUPT_FLAG.store(false, Ordering::Relaxed);
            
            // 读取中断相关数据
            let gpio_state = LAST_GPIO_STATE.load(Ordering::Relaxed);
            let interrupt_count = INTERRUPT_COUNT.load(Ordering::Relaxed);
            
            // 在这里处理业务逻辑（之前在ISR中的代码移到这里）
            println!("🚨 GPIO4 Interrupt #{} detected!", interrupt_count);
            println!("   └─ GPIO4 state: {}", if gpio_state { "HIGH" } else { "LOW" });
            
            // 这里可以添加你的具体业务逻辑，比如：
            // - 更新状态机
            // - 发送消息到队列
            // - 触发其他操作
            // - 复杂的计算或I/O操作
            
            println!("   └─ Business logic executed successfully");
        }
        
        // 主循环的正常工作
        println!("Hello world! Counter: {} (GPIO4 interrupt count: {})", 
                counter, 
                INTERRUPT_COUNT.load(Ordering::Relaxed));
        counter += 1;
        delay.delay_millis(1000);
    }
}