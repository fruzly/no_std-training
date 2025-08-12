// ADC - 电压器传感器

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    analog::adc::{Adc,AdcConfig, Attenuation}, 
    main,
};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();


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
    let pin_instance = peripherals.GPIO1;

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

    // 进入主循环
    loop {

        // 获取ADC读数
        let sample: u16 = nb::block!(adc1.read_oneshot(&mut adc1_pin)).unwrap();

        // 转换为电压
        // 原始值的范围是从0到4095（2^12 − 1），这是基于所选分辨率（12位）的ADC全量程。
        // 转换后的值则以毫伏（mV）为单位，显示从 0 到 ADC 可测量的最大电压。 最大值取决于设备的校准和前面描述的衰减设置。
        // 当前假设最大电压为3.3V。
        // 公式：实际值 = 原始读数 * 物理量最大值 / ADC最大读数
        let voltage: u32 = sample as u32 * 3300 / 4095;

        // 打印原始读数和转换后的电压
        println!(
            "Raw Reading: {}, Voltage Reading: {}mV",
            sample, voltage
        );

        // 在下次采样前等待半秒
        delay.delay_millis(500_u32);
    }
}