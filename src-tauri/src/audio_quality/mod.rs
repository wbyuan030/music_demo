pub mod metrics;
pub mod probe;
pub mod signal;

#[cfg(test)]
mod test {
    use rodio::{Decoder, Source};
    use std::io::Cursor;

    use super::metrics::{amplitude_jumps, harmonic_snr, silence_gaps};
    use super::signal::reference_sine_440hz_1s;

    #[test]
    fn wav_roundtrip() {
        let (wav_bytes, expected) = reference_sine_440hz_1s();
        let decoder = Decoder::new(Cursor::new(wav_bytes)).unwrap();
        let decoded: Vec<f32> = decoder.collect();

        // 样本数匹配
        assert_eq!(
            decoded.len(),
            expected.len(),
            "sample count mismatch: {} vs {}",
            decoded.len(),
            expected.len()
        );

        // 逐样本偏差
        let max_diff = decoded
            .iter()
            .zip(&expected)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(
            max_diff < 0.01,
            "max sample deviation too large: {}",
            max_diff
        );

        // 静音比例
        let silence_ratio = silence_ratio(&decoded);
        assert!(
            silence_ratio < 0.01,
            "silence ratio too high: {:.2}%",
            silence_ratio * 100.0
        );

        // NaN/Inf
        let bad = decoded
            .iter()
            .filter(|s| s.is_nan() || s.is_infinite())
            .count();
        assert_eq!(bad, 0, "found {} NaN/Inf samples", bad);

        // 额外指标（不断言，仅输出）
        let gaps = silence_gaps(&decoded, 44100);
        let jumps = amplitude_jumps(&decoded, 44100);
        let hnr = harmonic_snr(&decoded, 44100, 440.0);
        println!("wav_roundtrip: silence_gaps={}, amplitude_jumps={}, HNR={:.1}dB", gaps.len(), jumps.len(), hnr);
    }

    fn silence_ratio(samples: &[f32]) -> f64 {
        let silent = samples.iter().filter(|s| s.abs() < 1e-6).count();
        silent as f64 / samples.len() as f64
    }
}
