/// 静音段信息
#[derive(Debug, Clone)]
pub struct GapInfo {
    pub start_sec: f64,
    pub duration_sec: f64,
}

/// 振幅跳变信息
#[derive(Debug, Clone)]
pub struct JumpInfo {
    pub time_sec: f64,
    pub ratio: f64,
}

/// 静音段检测：滑动窗口(100ms)内 RMS < -60dBFS (0.001)，标记为 gap
pub fn silence_gaps(samples: &[f32], sample_rate: u32) -> Vec<GapInfo> {
    let window_size = (sample_rate as usize) / 10; // 100ms
    let threshold = 0.001f32; // -60dBFS

    if samples.len() < window_size {
        return Vec::new();
    }

    let mut gaps = Vec::new();
    let mut in_gap = false;
    let mut gap_start = 0usize;

    for chunk in samples.chunks(window_size) {
        let rms = (chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32).sqrt();
        let is_silent = rms < threshold;

        match (is_silent, in_gap) {
            (true, false) => {
                in_gap = true;
                gap_start = 0; // 相对位置，下面算绝对时间
            }
            (false, true) => {
                in_gap = false;
                // gap_start 是 chunk index，转成秒
            }
            _ => {}
        }
    }

    // 更精确的逐 sample 扫描
    let mut i = 0;
    while i < samples.len() {
        let end = (i + window_size).min(samples.len());
        let chunk = &samples[i..end];
        let rms = (chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32).sqrt();

        if rms < threshold && !in_gap {
            in_gap = true;
            gap_start = i;
        } else if rms >= threshold && in_gap {
            in_gap = false;
            let duration = (i - gap_start) as f64 / sample_rate as f64;
            if duration > 0.01 {
                // 过滤 < 10ms 的短间隙
                gaps.push(GapInfo {
                    start_sec: gap_start as f64 / sample_rate as f64,
                    duration_sec: duration,
                });
            }
        }
        i = end;
    }
    // 末尾如果还在 gap 中
    if in_gap {
        let duration = (samples.len() - gap_start) as f64 / sample_rate as f64;
        if duration > 0.01 {
            gaps.push(GapInfo {
                start_sec: gap_start as f64 / sample_rate as f64,
                duration_sec: duration,
            });
        }
    }

    gaps
}

/// 振幅跳变检测：连续 sample 差值 > 前一个窗口(100ms)最大振幅的 10 倍
pub fn amplitude_jumps(samples: &[f32], sample_rate: u32) -> Vec<JumpInfo> {
    let window_size = (sample_rate as usize) / 10; // 100ms
    let mut jumps = Vec::new();

    if samples.len() < window_size + 1 {
        return jumps;
    }

    let mut i = 0;
    while i + window_size < samples.len() - 1 {
        let window = &samples[i..i + window_size];
        let max_amp = window
            .iter()
            .map(|s| s.abs())
            .fold(0.0f32, f32::max)
            .max(1e-10); // 避免除零

        let diff = (samples[i + window_size] - samples[i + window_size - 1])
            .abs();
        let ratio = diff / max_amp;

        if ratio > 10.0 {
            jumps.push(JumpInfo {
                time_sec: (i + window_size) as f64 / sample_rate as f64,
                ratio: ratio as f64,
            });
        }
        i += window_size;
    }

    jumps
}

/// 谐波信噪比 (Harmonic-to-Noise Ratio)
/// 用 Goertzel 算法检测基频(440Hz)及其整数倍频点的能量
/// 返回值: dB, 越高越好 (> 20dB 良好, > 30dB CD 音质)
pub fn harmonic_snr(samples: &[f32], sample_rate: u32, fundamental_freq: f64) -> f64 {
    let harmonics = [1.0, 2.0, 3.0, 4.0]; // 基频 + 3 次谐波
    let total_rms = samples.iter().map(|s| (s * s) as f64).sum::<f64>();

    if total_rms < 1e-20 {
        return 0.0; // 静音
    }

    let mut harmonic_energy = 0.0f64;

    for &h in &harmonics {
        let freq = fundamental_freq * h;
        let (q0, q1) = goertzel(samples, freq, sample_rate);
        let magnitude_sq = q1 * q1 + q0 * q0 - q1 * q0 * goertzel_coeff(freq, sample_rate);
        harmonic_energy += magnitude_sq;
    }

    let noise_energy = total_rms - harmonic_energy;
    if noise_energy <= 0.0 {
        return 60.0; // 无噪声
    }

    let hnr = 10.0 * (harmonic_energy / noise_energy).log10();
    hnr.max(0.0) // 不低于 0dB
}

fn goertzel_coeff(freq: f64, sample_rate: u32) -> f64 {
    2.0 * (2.0 * std::f64::consts::PI * freq / sample_rate as f64).cos()
}

/// Goertzel 算法单频点检测，返回 (q0, q1) 用于计算幅度
fn goertzel(samples: &[f32], target_freq: f64, sample_rate: u32) -> (f64, f64) {
    let coeff = goertzel_coeff(target_freq, sample_rate);
    let mut q0 = 0.0f64;
    let mut q1 = 0.0f64;
    let mut q2 = 0.0f64;

    for &s in samples {
        q0 = coeff * q1 - q2 + s as f64;
        q2 = q1;
        q1 = q0;
    }

    (q0, q1)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn silence_gaps_on_sine() {
        // 1 秒正弦波，无静音段
        let sample_rate = 44100u32;
        let samples: Vec<f32> = (0..sample_rate as usize)
            .map(|i| {
                let t = i as f64 / sample_rate as f64;
                (2.0 * std::f64::consts::PI * 440.0 * t).sin() as f32
            })
            .collect();
        let gaps = silence_gaps(&samples, sample_rate);
        assert!(gaps.is_empty(), "expected no gaps, found: {:?}", gaps);
    }

    #[test]
    fn silence_gaps_detects_silence() {
        // 前半秒正弦波，后半秒静音
        let sample_rate = 44100u32;
        let mut samples = Vec::with_capacity(sample_rate as usize);
        for i in 0..sample_rate as usize / 2 {
            let t = i as f64 / sample_rate as f64;
            samples.push((2.0 * std::f64::consts::PI * 440.0 * t).sin() as f32);
        }
        samples.resize(sample_rate as usize, 0.0f32); // 后半秒静音

        let gaps = silence_gaps(&samples, sample_rate);
        assert!(!gaps.is_empty(), "expected at least one gap");
        if let Some(gap) = gaps.first() {
            assert!((gap.start_sec - 0.5).abs() < 0.15, "gap should start around 0.5s");
        }
    }

    #[test]
    fn amplitude_jumps_on_sine() {
        let sample_rate = 44100u32;
        let samples: Vec<f32> = (0..sample_rate as usize)
            .map(|i| {
                let t = i as f64 / sample_rate as f64;
                (2.0 * std::f64::consts::PI * 440.0 * t).sin() as f32
            })
            .collect();
        let jumps = amplitude_jumps(&samples, sample_rate);
        assert!(jumps.is_empty(), "expected no jumps on pure sine");
    }

    #[test]
    fn amplitude_jumps_detects_spike() {
        let sample_rate = 44100u32;
        let mut samples: Vec<f32> = (0..sample_rate as usize)
            .map(|i| {
                let t = i as f64 / sample_rate as f64;
                (2.0 * std::f64::consts::PI * 440.0 * t).sin() as f32
            })
            .collect();
        // 在 0.5 秒处注入一个尖峰
        let idx = sample_rate as usize / 2;
        samples[idx] = 100.0;

        let jumps = amplitude_jumps(&samples, sample_rate);
        assert!(!jumps.is_empty(), "expected at least one jump");
    }

    #[test]
    fn harmonic_snr_on_sine() {
        let sample_rate = 44100u32;
        let samples: Vec<f32> = (0..sample_rate as usize)
            .map(|i| {
                let t = i as f64 / sample_rate as f64;
                (2.0 * std::f64::consts::PI * 440.0 * t).sin() as f32
            })
            .collect();
        let hnr = harmonic_snr(&samples, sample_rate, 440.0);
        assert!(hnr > 20.0, "HNR should be > 20dB for pure sine, got {:.1}dB", hnr);
    }
}
