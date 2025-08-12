// PWM - LED 呼吸灯 - 使用硬件渐变功能 start_duty_fade

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
        duty: ledc::timer::config::Duty::Duty14Bit, // 分辨率：14-bit
        clock_source: ledc::timer::LSClockSource::APBClk,
        frequency: Rate::from_khz(1), // 频率：1KHz
    }).unwrap();

    let mut channel = ledc.channel(channel::Number::Channel0, led_pin);
    channel.configure(channel::config::Config {
        timer: &timer,
        duty_pct: 0,
        pin_config: channel::config::PinConfig::PushPull,
    }).unwrap();

    // 创建一个延时器实例，用于后续的延时操作
    let delay = esp_hal::delay::Delay::new();
    let fade_time_ms = 2000_u16; // 渐变时间: 2秒

    // 进入主循环
    loop {
        // 1. 开始从 0% -> 100% 的硬件渐变
        // 这个函数会立即返回，硬件（ESP32-S3 的 LEDC（LED控制器））在后台开始工作，但不会阻塞当前线程
        // 硬件在后台自动执行从 0% 到 100% 的 PWM 占空比渐变，这个过程与 CPU 执行代码是并行进行的
        channel.start_duty_fade(0, 100, fade_time_ms).unwrap();
        
        // 2. 手动同步软硬件节奏，等待硬件渐变完成，确保呼吸灯效果的正确节奏
        // 因为 start_duty_fade 是非阻塞的, 这里的 delay 是为了同步软件和硬件的节奏，等待上一个硬件任务完成后再发布下一个。
        delay.delay_millis(fade_time_ms as u32);

        // 3. 开始从 100% -> 0% 的硬件渐变
        channel.start_duty_fade(100, 0, fade_time_ms).unwrap();

        // 4. 再次等待
        delay.delay_millis(fade_time_ms as u32);
    }
}
