#![allow(dead_code, unused)]
use microfft::real::rfft_1024;
use microfft::Complex32;

pub const SAMPLE_RATE: f32 = 21300.0;
pub const FFT_SIZE: usize = 1024;
pub const NOISE_FLOOR: f32 = 1_717_401.0;

pub struct FftResult {
    pub peak_power: f32,
    pub total_power: f32,
    pub ratio: f32,
    pub is_detected: bool,
}

pub fn analyze(samples: &mut [f32; 1024]) -> (FftResult, [Complex32; 512]) {
    let spectrum = rfft_1024(samples);

    let target_bin = (1976.0 * FFT_SIZE as f32 / SAMPLE_RATE) as usize;

    let total_power: f32 =
        spectrum.iter().map(|c| c.norm_sqr()).sum::<f32>() / spectrum.len() as f32;

    let mut peak_power = 0.0f32;
    for i in target_bin.saturating_sub(3)..=(target_bin + 3).min(spectrum.len() - 1) {
        let power = spectrum[i].norm_sqr();
        if power > peak_power {
            peak_power = power;
        }
    }

    let ratio = peak_power / total_power;
    let is_detected = peak_power > NOISE_FLOOR * 2.5 && ratio > 0.001;

    let result = FftResult {
        peak_power,
        total_power,
        ratio,
        is_detected,
    };

    (result, *spectrum)
}
