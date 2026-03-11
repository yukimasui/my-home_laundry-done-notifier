#![allow(dead_code, unused)]
use anyhow::Result;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Rectangle},
    text::Text,
};
use embedded_hal::i2c::I2c;
use num_complex::Complex32;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::{prelude::*, Ssd1306};

pub type Display<I> =
    Ssd1306<I2CInterface<I>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>;

pub fn draw_spectrum<I: I2c>(display: &mut Display<I>, spectrum: &[Complex32]) -> Result<()> {
    let bins = spectrum.len();
    let mut prev_y: Option<i32> = None;

    for x in 0..128usize {
        let bin_index = x * bins / 128;
        let power = spectrum[bin_index].norm_sqr();
        let normalized = (power.log10().max(0.0) / 10.0).min(1.0);
        let y = 64 - (normalized * 40.0) as i32;

        if let Some(py) = prev_y {
            Line::new(Point::new(x as i32 - 1, py), Point::new(x as i32, y))
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                .draw(display)
                .map_err(|_| anyhow::anyhow!("Draw error"))?;
        }
        prev_y = Some(y);
    }
    Ok(())
}

pub fn draw_status<I: I2c>(display: &mut Display<I>, peak_power: f32, ration: f32) -> Result<()> {
    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let peak_value_text = format!("PeakPower:{}\nRatio:{}", peak_power, ration);
    Text::new(&peak_value_text, Point::new(0, 10), text_style)
        .draw(display)
        .map_err(|_| anyhow::anyhow!("Draw error"))?;
    Ok(())
}
