// 循环

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    gpio::{Input, InputConfig, Io, Level, Output, OutputConfig, Pull},
    main, timer::timg::TimerGroup,
};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // 获取外设工具箱
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // 禁用看门狗定时器
    let timer_group0 = TimerGroup::new(peripherals.TIMG0);
    let mut wdt0 = timer_group0.wdt;
    wdt0.disable();
    
    let timer_group1 = TimerGroup::new(peripherals.TIMG1);
    let mut wdt1 = timer_group1.wdt;
    wdt1.disable();
    
    println!("Watchdog timers disabled!");

    // 创建 GPIO 总管家
    let _io = Io::new(peripherals.IO_MUX);

    // 配置 LED：将 gpio4 设置为输出引脚，初始状态为高电平
    let mut led = Output::new(peripherals.GPIO4, Level::High, OutputConfig::default());

    let config = InputConfig::default().with_pull(Pull::Up);
    // 配置按钮：将 gpio9 设置为输入引脚，并启用内部上拉电阻
    let button = Input::new(peripherals.GPIO9, config);
    println!("button level {:?} - {:?} - {:#?}", button.is_low(), button.is_high(), button.level());

    // 创建一个可变的延时变量，初始值很大，代表初始闪烁速度很慢
    // 软件延时 (Software Delay): 通过让 CPU 执行一个耗时的循环来达到延时效果。
    // 这种方法简单，但效率极低，因为在延时期间 CPU 被完全被“空转”占用，无法做任何其他事情。
    let mut blinkdelay = 1_000_000_u32;

    // 先将 LED 熄灭，确保初始状态一致
    led.set_low();

    // 进入主循环
    loop {
        // 这是一个软件延时循环。它会空转 `blinkdelay` 次。
        // 在这个延时期间，它不停地检查按钮状态。
        for _i in 1..blinkdelay {
            // println!("{} - Button: {}", i, button.is_low());
            // is_low() 检查按钮是否被按下（因为我们接到了GND，并且用了上拉电阻）
            if button.is_low() {
                println!("Button pressed!");

                // 如果按下了，就减少延时值，让下一次闪烁变得更快
                blinkdelay = blinkdelay - 25_000_u32;
                // 如果延时值变得太小，就把它重置回初始的大值，实现循环调速
                if blinkdelay < 25_000 {
                    blinkdelay = 1_000_000_u32;
                }
            }
        }
        // 延时循环结束后（也就是过了一段时间后），翻转 LED 的状态
        led.toggle();
    }
}
