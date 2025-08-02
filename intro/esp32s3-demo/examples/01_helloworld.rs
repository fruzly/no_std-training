
#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{delay::Delay, main, time::Duration, timer::timg::{MwdtStage, TimerGroup}};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

// 1. 正确配置和喂看门狗（开发环境）
fn dev_watchdog_disabled() -> ! {
    println!("\n=== ESP32-S3 Rust Application Starting ===");
    println!("Chip: ESP32-S3");
    println!("Build: Release - Watchdog Disabled");
    
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // 禁用看门狗定时器
    let timer_group0 = TimerGroup::new(peripherals.TIMG0);
    let mut wdt0 = timer_group0.wdt;
    wdt0.disable();
    
    let timer_group1 = TimerGroup::new(peripherals.TIMG1);
    let mut wdt1 = timer_group1.wdt;
    wdt1.disable();
    
    println!("Watchdog timers disabled!");
    
    let delay = Delay::new();
    delay.delay_millis(1000);
    
    println!("=== System Ready ===");
    println!("Starting counter loop with 1 second interval");
    
    let mut counter = 0;
    
    loop {
        println!("[Counter: {}] Hello world! - System running stable without watchdog", counter);
        
        if counter > 0 && counter % 10 == 0 {
            println!("  -> Milestone: {} iterations completed successfully!", counter);
        }
        
        counter += 1;
        delay.delay_millis(1000);
    }
}


// 2. 正确配置和喂看门狗（生产环境推荐）
fn prod_watchdog_managed() -> ! {
    println!("\n=== ESP32-S3 Rust Application Starting ===");
    println!("Chip: ESP32-S3");
    println!("Build: Release - Watchdog Managed");
    
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // 配置看门狗定时器，设置更长的超时时间
    let timer_group0 = TimerGroup::new(peripherals.TIMG0);
    let mut wdt0 = timer_group0.wdt;
    
    // 启用看门狗，设置10秒超时
    wdt0.enable();
    wdt0.set_timeout(MwdtStage::Stage0, Duration::from_secs(10)); // 10秒超时
    
    println!("Watchdog configured with 10s timeout");
    
    let delay = Delay::new();
    delay.delay_millis(1000);
    
    println!("=== System Ready ===");
    println!("Starting counter loop with watchdog feeding");
    
    let mut counter = 0;
    
    loop {
        // 喂看门狗 - 重置看门狗定时器
        wdt0.feed();
        
        println!("[Counter: {}] Hello world! - Watchdog fed", counter);
        
        if counter > 0 && counter % 5 == 0 {
            println!("  -> Watchdog status: Active, last fed at counter {}", counter);
        }
        
        counter += 1;
        
        // 安全的延迟方式：分段延迟并定期喂狗
        for i in 0..10 {
            delay.delay_millis(100); // 100ms * 10 = 1000ms
            if i % 3 == 0 {
                wdt0.feed(); // 每300ms喂一次狗
            }
        }
    }
}


// 3. 使用任务调度器方式（更高级）
fn advanced_cooperative_scheduling() -> ! {
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

#[main]
fn main() -> ! {
    // dev_watchdog_disabled();
    // prod_watchdog_managed();
    advanced_cooperative_scheduling();
    loop {}
}