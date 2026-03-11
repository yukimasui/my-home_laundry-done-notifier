mod display;
mod fft;
mod mqtt;

use anyhow::Result;
use core::cell::RefCell;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
};
use embedded_hal_bus::i2c::RefCellDevice;
use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::oneshot::*;
use esp_idf_hal::i2c::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use std::collections::VecDeque;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;
    // i2cの設定
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;
    let config = I2cConfig::new().baudrate(400.kHz().into());
    let i2c_driver = I2cDriver::new(i2c, sda, scl, &config)?;

    // 複数のi2c機器を使えるようにする
    let i2c_ref_cell = RefCell::new(i2c_driver);

    let display_i2c = RefCellDevice::new(&i2c_ref_cell);

    // ディスプレイのセットアップ
    let interface = I2CDisplayInterface::new(display_i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().unwrap();

    let adc = AdcDriver::new(peripherals.adc1)?;

    let config = AdcChannelConfig {
        attenuation: DB_11,
        ..Default::default()
    };
    let mut adc_pin = AdcChannelDriver::new(&adc, peripherals.pins.gpio34, &config)?;

    let mut window: VecDeque<bool> = VecDeque::with_capacity(10);
    let modem = peripherals.modem;
    let mut detected_count = 0;

    loop {
        // FFT部分
        let mut samples = [0f32; fft::FFT_SIZE];
        for s in samples.iter_mut() {
            *s = adc.read(&mut adc_pin)? as f32;
        }

        let (result, spectrum) = fft::analyze(&mut samples);
        let is_detected = result.is_detected;

        if window.len() >= 10 {
            window.pop_front();
        }
        window.push_back(is_detected);

        let count = window.iter().filter(|&&x| x).count();
        let window_detected = count >= 7;

        // 判定
        if window_detected {
            // 一度検知されたらスライドウィンドウをクリア
            window.clear();
            detected_count += 1;
            println!("★ブザー検知★");
            Circle::new(Point::new(10, 20), 20)
                .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                .draw(&mut display)
                .map_err(|_| anyhow::anyhow!("Draw error"))?;
        }

        if detected_count >= 4 {
            break;
        }

        display
            .clear(BinaryColor::Off)
            .map_err(|_| anyhow::anyhow!("Clear error"))?;

        display::draw_spectrum(&mut display, &spectrum)?;
        display::draw_status(&mut display, result.peak_power, result.ratio)?;

        display
            .flush()
            .map_err(|_| anyhow::anyhow!("Display flush error"))?;
    }

    // std::thread::sleep(std::time::Duration::from_millis(5));
    // send_mqtt_notification(modem)?;
    mqtt::send_mqtt_notification(modem)?;
    Ok(())
}

// fn device_names(i2c_driver: &mut I2cDriver<'_>) {
//     println!("I2Cバスをスキャン中...");
//     for address in 0x03..0x78 {
//         if i2c_driver.write(address, &[], 1000).is_ok() {
//             println!("デバイスが見つかりました: 0x{:02X}", address);
//         }
//     }
// }
