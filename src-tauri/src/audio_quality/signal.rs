#![allow(dead_code)]
/// 生成 1 秒 440Hz 正弦波 @ 44100Hz mono 的 WAV 文件字节
/// 返回值: (wav_bytes, expected_f32_samples)
pub fn reference_sine_440hz_1s() -> (Vec<u8>, Vec<f32>) {
    let sample_rate = 44100u32;
    let num_samples = sample_rate as usize; // 1 秒
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 32; // f32
    let bytes_per_sample = (bits_per_sample / 8) as u32;
    let data_size = num_samples as u32 * bytes_per_sample;

    // 生成样本
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let v = (2.0 * std::f64::consts::PI * 440.0 * t).sin() as f32;
        samples.push(v);
    }

    // 手拼 WAV header (44 bytes)
    let mut wav = Vec::with_capacity(44 + data_size as usize);

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_size).to_le_bytes()); // chunk size
    wav.extend_from_slice(b"WAVE");

    // fmt subchunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // subchunk size (PCM)
    wav.extend_from_slice(&3u16.to_le_bytes());  // audio format (IEEE float = 3)
    wav.extend_from_slice(&num_channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&(sample_rate * bytes_per_sample).to_le_bytes()); // byte rate
    wav.extend_from_slice(&(bytes_per_sample as u16).to_le_bytes()); // block align
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data subchunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    for v in &samples {
        wav.extend_from_slice(&v.to_le_bytes());
    }

    (wav, samples)
}
