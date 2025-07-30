// #![no_std]
// #![no_main]

// use esp_backtrace as _;
// use esp_hal::{delay::Delay, main};
// use esp_println::println;

// esp_bootloader_esp_idf::esp_app_desc!();

// #[main]
// fn main() -> ! {
//     // 使用esp_println直接输出，不依赖log系统
//     println!("ESP32-S3 Rust Application Starting!");
    
//     let delay = Delay::new();
//     delay.delay_millis(1000); // 给系统一些时间稳定
    
//     println!("Entering main loop...");
    
//     let mut counter = 0;
//     loop {
//         println!("Hello world! Counter: {}", counter);
//         counter += 1;
//         delay.delay_millis(1000);
//     }
// }



// #![no_std]
// #![no_main]

// use esp_backtrace as _;
// use esp_hal::{delay::Delay, main};
// use esp_println::println;

// esp_bootloader_esp_idf::esp_app_desc!();

// #[main]
// fn main() -> ! {
//     println!("\n=== ESP32-S3 Rust Application Starting ===");
//     println!("Chip: ESP32-S3");
//     println!("Build: Release");
    
//     let delay = Delay::new();
//     delay.delay_millis(1000);
    
//     println!("=== System Ready ===");
//     println!("Starting counter loop with 1 second interval");
    
//     let mut counter = 0;
//     let mut last_time = 0u32; // 简单的时间跟踪
    
//     loop {
//         let current_time = last_time + 1;
//         println!("[{}s] Hello world! Counter: {} - Status: OK", current_time, counter);
        
//         // 检查是否接近重启周期
//         if counter > 0 && counter % 5 == 0 {
//             println!("  -> Reached counter {}, checking system stability...", counter);
//             delay.delay_millis(100);
//             println!("  -> System check passed, continuing...");
//         }
        
//         counter += 1;
//         last_time = current_time;
//         delay.delay_millis(900); // 900ms + 上面的延迟 ≈ 1000ms
//     }
// }

// 如上代码，开发板会有周期性的重启问题，解决如下

// 1. 禁用看门狗：成功修复，但是适合调试

// #![no_std]
// #![no_main]

// use esp_backtrace as _;
// use esp_hal::{delay::Delay, main, timer::timg::TimerGroup};
// use esp_println::println;

// esp_bootloader_esp_idf::esp_app_desc!();

// #[main]
// fn main() -> ! {
//     println!("\n=== ESP32-S3 Rust Application Starting ===");
//     println!("Chip: ESP32-S3");
//     println!("Build: Release - Watchdog Disabled");
    
//     let peripherals = esp_hal::init(esp_hal::Config::default());
    
//     // 禁用看门狗定时器
//     let timer_group0 = TimerGroup::new(peripherals.TIMG0);
//     let mut wdt0 = timer_group0.wdt;
//     wdt0.disable();
    
//     let timer_group1 = TimerGroup::new(peripherals.TIMG1);
//     let mut wdt1 = timer_group1.wdt;
//     wdt1.disable();
    
//     println!("Watchdog timers disabled!");
    
//     let delay = Delay::new();
//     delay.delay_millis(1000);
    
//     println!("=== System Ready ===");
//     println!("Starting counter loop with 1 second interval");
    
//     let mut counter = 0;
    
//     loop {
//         println!("[Counter: {}] Hello world! - System running stable without watchdog", counter);
        
//         if counter > 0 && counter % 10 == 0 {
//             println!("  -> Milestone: {} iterations completed successfully!", counter);
//         }
        
//         counter += 1;
//         delay.delay_millis(1000);
//     }
// }


// 2. 正确配置和喂看门狗（生产环境推荐）

// #![no_std]
// #![no_main]

// use esp_backtrace as _;
// use esp_hal::{delay::Delay, main, time::Duration, timer::timg::{MwdtStage, TimerGroup}};
// use esp_println::println;

// esp_bootloader_esp_idf::esp_app_desc!();

// #[main]
// fn main() -> ! {
//     println!("\n=== ESP32-S3 Rust Application Starting ===");
//     println!("Chip: ESP32-S3");
//     println!("Build: Release - Watchdog Managed");
    
//     let peripherals = esp_hal::init(esp_hal::Config::default());
    
//     // 配置看门狗定时器，设置更长的超时时间
//     let timer_group0 = TimerGroup::new(peripherals.TIMG0);
//     let mut wdt0 = timer_group0.wdt;
    
//     // 启用看门狗，设置10秒超时
//     wdt0.enable();
//     wdt0.set_timeout(MwdtStage::Stage0, Duration::from_secs(10)); // 10秒超时
    
//     println!("Watchdog configured with 10s timeout");
    
//     let delay = Delay::new();
//     delay.delay_millis(1000);
    
//     println!("=== System Ready ===");
//     println!("Starting counter loop with watchdog feeding");
    
//     let mut counter = 0;
    
//     loop {
//         // 喂看门狗 - 重置看门狗定时器
//         wdt0.feed();
        
//         println!("[Counter: {}] Hello world! - Watchdog fed", counter);
        
//         if counter > 0 && counter % 5 == 0 {
//             println!("  -> Watchdog status: Active, last fed at counter {}", counter);
//         }
        
//         counter += 1;
        
//         // 安全的延迟方式：分段延迟并定期喂狗
//         for i in 0..10 {
//             delay.delay_millis(100); // 100ms * 10 = 1000ms
//             if i % 3 == 0 {
//                 wdt0.feed(); // 每300ms喂一次狗
//             }
//         }
//     }
// }


// 3. 使用任务调度器方式（更高级）

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{delay::Delay, main, time::Duration, timer::timg::{MwdtStage, TimerGroup}};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    println!("\n=== ESP32-S3 Rust Application Starting ===");
    println!("Chip: ESP32-S3");
    println!("Build: Release - Cooperative Scheduling");
    
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // 配置看门狗
    let timer_group0 = TimerGroup::new(peripherals.TIMG0);
    let mut wdt0 = timer_group0.wdt;
    wdt0.enable();
    // 修改看门狗配置
    wdt0.set_timeout(MwdtStage::Stage0, Duration::from_secs(20)); // 20秒超时
    
    println!("Watchdog configured with 20s timeout");
    
    let delay = Delay::new();
    delay.delay_millis(1000);
    
    println!("=== System Ready ===");
    println!("Starting cooperative main loop");
    
    let mut counter = 0;
    
    loop {
        // 每10次循环喂一次看门狗
        if counter % 10 == 0 {
            wdt0.feed();
            println!("  -> Watchdog fed at counter {}", counter);
        }
        
        println!("[Counter: {}] Hello world! - Optimized scheduling", counter);
        
        // 优化延迟：单一1秒延迟，但分段喂狗如果需要
        delay.delay_millis(1000);
        
        counter += 1;
        
        if counter % 50 == 0 {
            println!("=== Long term stability test: {} iterations ===", counter);
        }
    }
}