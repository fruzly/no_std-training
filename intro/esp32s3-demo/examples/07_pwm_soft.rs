// 使用GPIO和延迟模拟PWM (软件PWM)

#![no_std]
#![no_main]

use embedded_hal::delay::DelayNs;
use esp_backtrace as _;
use esp_hal::{
    delay::Delay, gpio::{self, DriveMode, Level, Output, OutputConfig, Pull}, ledc::{self, channel::{self, ChannelIFace}, 
    timer::TimerIFace, Ledc, LowSpeed}, main, time::Rate 
};

esp_bootloader_esp_idf::esp_app_desc!();

// 软件PWM函数
// - pin: 可变的GPIO输出引脚
// - intensity: 亮度/占空比 (0-100)
// - frequency_hz: PWM信号频率 (Hz)
// - duration_ms: 该PWM信号持续的总时间 (ms)
fn soft_pwm(
    pin: &mut Output,
    intensity: u8,
    frequency_hz: u32,
    duration_ms: u32,
    delay: &mut Delay,
) {
    if frequency_hz == 0 {
        return;
    }
    // 1. 计算周期和高/低电平时间 (单位: 微秒)
    let period_us = 1_000_000 / frequency_hz;
    let on_time_us = (period_us as u64 * intensity as u64 / 100) as u32;
    let off_time_us = period_us - on_time_us;

    // 2. 计算需要循环的次数
    let cycles = duration_ms * 1000 / period_us;

    // 3. 循环产生PWM波形
    for _ in 0..cycles {
        if on_time_us > 0 {
            pin.set_high();
            delay.delay_us(on_time_us);
        }
        if off_time_us > 0 {
            pin.set_low();
            delay.delay_us(off_time_us);
        }
    }
    // 确保函数结束时引脚为低电平
    pin.set_low();
}

#[main]
fn main() -> ! {
    // 获取外设工具箱
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Create output pin configuration
    let led_pin_conf = OutputConfig::default()
        .with_drive_mode(DriveMode::PushPull)
        .with_pull(Pull::None);

    let mut led_pin = Output::new(peripherals.GPIO7, Level::Low, led_pin_conf);

    // 创建一个延时器实例，用于后续的延时操作
    let mut delay = esp_hal::delay::Delay::new();

    let max_intensity = 100_u8;
    let min_intensity = 0_u8;
    let pwm_frequency = 500; // 500Hz, 避免闪烁
    
    // 进入主循环
    loop {
        // 渐亮: 从 0% -> 100%
        for i in min_intensity..max_intensity {
            // 每次只输出一小段时间(20ms), 拼接成渐变效果
            soft_pwm(&mut led_pin, i, pwm_frequency, 20, &mut delay);
        }

        // 渐暗: 从 100% -> 0%
        for i in (min_intensity..max_intensity).rev() {
            soft_pwm(&mut led_pin, i, pwm_frequency, 20, &mut delay);
        }
    }
}
