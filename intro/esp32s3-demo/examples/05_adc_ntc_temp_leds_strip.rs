// ADC - NTC：温度控制一个 LED 灯的闪烁频率

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    analog::adc::{Adc,AdcConfig, Attenuation}, gpio::{Level, Output, OutputConfig}, main,
};
use esp_println::println;
use libm::log; // 用于计算自然对数

esp_bootloader_esp_idf::esp_app_desc!();


// 将一个范围映射到另一个范围的函数
fn map(
    x: u32,
    in_min: u32,
    in_max: u32,
    out_min: u32,
    out_max: u32,
) -> u32 {
    (x - in_min) * (out_max - out_min)
    / (in_max - in_min)
    + out_min
}

#[main]
fn main() -> ! {
    // 获取外设工具箱
    let peripherals = esp_hal::init(esp_hal::Config::default());
    // 直接获取GPIO外设，此处完全正确，原因如下：
    // 1. ADC驱动会接管引脚的模拟配置
    // 2. 不需要数字GPIO的上拉/下拉配置
    // 3. adc1_config.enable_pin() 会处理所有ADC相关的硬件设置
    // 同时也要注意：ESP32-S3 ADC 支持的引脚：
    // ADC1: GPIO1-GPIO10
    // ADC2: GPIO11-GPIO20
    let pin_instance = peripherals.GPIO4;

    // 创建ADC配置参数的实例
    let mut adc1_config = AdcConfig::new();
    // 通过ADC配置来处理引脚设置
    let mut adc1_pin = adc1_config.enable_pin(
        pin_instance,       // 原始GPIO外设：将此引脚作为模拟输入
        Attenuation::_11dB  // ADC专用配置：设置衰减值，ADC的测量范围将被扩大到可以测量0-3.3V的电压。
    );

    // 创建ADC驱动
    let mut adc1 = Adc::new(peripherals.ADC1, adc1_config);

    let delay = esp_hal::delay::Delay::new();

    const B: f64 = 3950.0; // NTC 热敏电阻的B值，从数据手册查得
    const VMAX: f64 = 4095.0; // 12位 ADC 最大原始读数，即 2^12 - 1

    // let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

    // 定义一个合理的温度范围，比如0到30摄氏度
    const MIN_TEMP: u32 = 0;
    const MAX_TEMP: u32 = 30;

    // 定义对应的延时范围，注意是反向的
    // 低温对应长延时（慢闪），高温对应短延时（快闪）
    const MAX_DELAY: u32 = 1000; // 1000ms
    const MIN_DELAY: u32 = 100;  // 100ms

    // 将3个LED引脚配置为输出，并放入一个数组中
    let mut leds = [
        Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO6, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO7, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default()),
    ];
    let leds_len = leds.len();

    // 计算NTC电阻值
    // 进入主循环
    loop {

        // 获取ADC读数
        let sample: u16 = nb::block!(adc1.read_oneshot(&mut adc1_pin)).unwrap();

        // 转换温度
        let temperature = 1.0
            / (log(1.0 / (VMAX / sample as f64 - 1.0)) / B
            + 1.0 / 298.15) // 298.15K 即 25°C
            - 273.15; // 273.15 这个值是用来将开尔文温度结果转换为摄氏度的

        let temp_u32 = temperature as u32;

        // 将温度范围 [10, 25] 映射到要点亮的LED数量 [0, leds_len]
        let leds_to_light = map(temp_u32, MIN_TEMP, MAX_TEMP, 0, leds_len as u32);
        println!("Temperature: {}C, LEDs to light: {}", temp_u32, leds_to_light);
        // 根据计算结果控制LED灯条
        for i in 0..leds_len {
            if i < leds_to_light as usize {
                leds[i].set_high(); // 点亮
            } else {
                leds[i].set_low();  // 熄灭
            }
        }

        delay.delay_millis(200_u32);
    }
}