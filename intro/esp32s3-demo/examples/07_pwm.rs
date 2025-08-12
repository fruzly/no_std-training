// PWM - LED 呼吸灯

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    ledc::{self, channel::{self, ChannelIFace}, timer::TimerIFace, Ledc, LowSpeed},
    main,
    time::Rate,
};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // 获取外设工具箱
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    let led_pin = peripherals.GPIO7;

    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(ledc::LSGlobalClkSource::APBClk);

    let mut timer = ledc.timer::<LowSpeed>(ledc::timer::Number::Timer0);
    timer.configure(ledc::timer::config::Config {
        duty: ledc::timer::config::Duty::Duty14Bit, // 分辨率：10-bit
        clock_source: ledc::timer::LSClockSource::APBClk,
        frequency: Rate::from_khz(1), // 频率：50KHz
    }).unwrap();

    let mut channel = ledc.channel(channel::Number::Channel0, led_pin);
    channel.configure(channel::config::Config {
        timer: &timer,
        duty_pct: 0,
        pin_config: channel::config::PinConfig::PushPull,
    }).unwrap();

    // 创建一个延时器实例，用于后续的延时操作
    let delay = esp_hal::delay::Delay::new();

    // 进入主循环
    loop {
        // 从暗到亮：将占空比从0%逐渐增加到100%
        for duty_pct in 0..=100 {
            // 调用 set_duty_pct 方法设置新的占空比
            channel.set_duty(duty_pct).unwrap();
            // 短暂延时，使亮度变化可见
            delay.delay_millis(10);
        }

        // 从亮到暗：将占空比从100%逐渐减小到0%
        for duty_pct in (0..=100).rev() {
            // 设置新的占空比
            channel.set_duty(duty_pct).unwrap();
            // 短暂延时
            delay.delay_millis(10);
        }
    }
}
