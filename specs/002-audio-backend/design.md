# Design: 音频质量量化测试体系

**Created**: 2026-07-01

**Status**: Approved

**Context**: `specs/002-audio-backend/spec.md` — 音频输出颗粒感问题调查

**Goal**: 建立可量化的测试体系，在不改变现有音频管线的条件下，检测和度量解码/播放过程中的质量问题，为后续的音频后端方案选型提供数据支撑。

---

## 1. Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    音频测试体系                           │
├─────────────────┬──────────────────┬────────────────────┤
│  离线: WAV往返   │  离线: 真实音频    │  在线: 播放探针      │
│  (signal.rs)    │  (metrics.rs)     │  (probe.rs)        │
├─────────────────┼──────────────────┼────────────────────┤
│ 参考信号生成      │ 静音段检测         │ 欠载事件记录          │
│ 逐样本对比        │ 振幅跳变检测       │ gap 时间序列          │
│ NaN/Inf 检测     │ 谐波信噪比(HSR)    │ 严重欠载标记          │
├─────────────────┼──────────────────┼────────────────────┤
│  CI可跑 (无设备)  │  CI可跑 (缓存文件)  │  需要真实音频设备       │
└─────────────────┴──────────────────┴────────────────────┘

三层各自独立，共享 metrics.rs 的纯函数。
```

---

## 2. Module: `audio_quality`

### 2.1 File layout

```
src-tauri/src/audio_quality/
├── mod.rs           # pub mod signal, metrics, probe; test 入口
├── signal.rs        # 参考信号生成
├── metrics.rs       # 纯函数：样本级质量指标
└── probe.rs         # 在线探针 struct
```

### 2.2 Dependencies

- `signal.rs`: 无外部依赖（手拼 WAV header + 数学运算）
- `metrics.rs`: `std::f32`, `std::f64` 数学运算，无外部依赖
- `probe.rs`: `std::time::Duration`, `tracing`（可选，用于 warn! 严重欠载）

不引入新 Cargo 依赖。不改变现有 `music_handler` 模块的公开 API。

---

## 3. signal.rs — 参考信号生成

### 3.1 Interface

```rust
/// 生成 1 秒 440Hz 正弦波 @ 44100Hz mono 的 WAV 文件字节
/// 返回值: (wav_bytes, expected_f32_samples)
pub fn reference_sine_440hz_1s() -> (Vec<u8>, Vec<f32>)
```

### 3.2 WAV header

固定 44 字节 PCM header: `RIFF` chunk + `fmt ` subchunk (PCM=1, mono=1, 44100Hz, 32-bit float) + `data` subchunk。样本值用 `f32::to_le_bytes()` 写入。

### 3.3 Expected samples

`expected[i] = sin(2π × 440 × i / 44100)`，共 44100 个 f32。与 WAV 字节内的 sample 值完全相同。

---

## 4. metrics.rs — 样本级指标

### 4.1 Interface

```rust
/// 静音段：滑动窗口(100ms)内 RMS < 阈值(-60dBFS = 0.001)，标记为 gap
pub fn silence_gaps(samples: &[f32], sample_rate: u32) -> Vec<GapInfo>
pub struct GapInfo { pub start_sec: f64, pub duration_sec: f64 }

/// 振幅跳变：连续 sample 差值 > 前一个窗口(100ms)最大振幅的 10 倍
pub fn amplitude_jumps(samples: &[f32], sample_rate: u32) -> Vec<JumpInfo>
pub struct JumpInfo { pub time_sec: f64, pub ratio: f64 }

/// 谐波信噪比 (Harmonic-to-Noise Ratio)
/// HNR = 10 * log10(harmonic_energy / noise_energy)
/// harmonic_energy = 基频(440Hz)及其整数倍频段的能量
/// noise_energy = 总能量 - harmonic_energy
/// 返回值越高越好，> 20dB 为良好
pub fn harmonic_snr(samples: &[f32], sample_rate: u32, fundamental_freq: f64) -> f64
```

### 4.2 Algorithm notes

**谐波信噪比**：用 Goertzel 算法逐频点检测（比 FFT 更省内存，只需几个频率点），检查 440Hz, 880Hz, 1320Hz, 1760Hz 的幅度。harmonic_energy = 这些频点能量之和。noise_energy = 总 RMS - harmonic_energy。

阈值参考：CD 音质 HNR > 30dB；AMR 窄带通常在 10-15dB。

---

## 5. probe.rs — 在线播放探针

### 5.1 Interface

```rust
pub struct PlaybackProbe {
    last_pos: Duration,
    last_wall_clock: Instant,
    total_elapsed: f64,
    stall_threshold: Duration,     // default: 100ms
    severe_threshold: Duration,    // default: 500ms
    pub stall_events: Vec<StallEvent>,
    pub max_gap: Duration,
}

pub struct StallEvent {
    pub elapsed_sec: f64,   // 从播放开始到欠载的时间
    pub gap: Duration,       // 期望位置与实际位置的差值
}

impl PlaybackProbe {
    pub fn new() -> Self;
    pub fn tick(&mut self, current_pos: Duration);
    pub fn report(&self) -> ProbeReport;
}

pub struct ProbeReport {
    pub stall_count: usize,
    pub severe_count: usize,
    pub max_gap: Duration,
    pub avg_gap: Duration,
    pub stall_timeline: Vec<(f64, Duration)>,  // (elapsed_sec, gap)
}
```

### 5.2 Integration point

在 `handler.rs` 的 `spawn_progress` 线程中：

```rust
// 现有代码 (每 500ms):
let pos = sink.get_pos();
app_handle.emit("play_progress", pos);

// 新增:
probe.tick(pos);   // ← 一行注入
```

播放开始（`MusicState::Play`）时创建新 `probe`，播放结束（`play_end`）时 emit `probe.report()` 到前端 console。

### 5.3 Tick logic

```rust
fn tick(&mut self, current_pos: Duration) {
    let elapsed = self.last_wall_clock.elapsed();  // std::time::Instant
    let expected = self.last_pos + elapsed;          // 从上次位置推进 elapsed
    let actual = current_pos;
    let gap = expected.checked_sub(actual).unwrap_or(Duration::ZERO);

    if gap > self.stall_threshold {
        self.stall_events.push(StallEvent { elapsed_sec: self.total_elapsed, gap });
        if gap > self.severe_threshold {
            tracing::warn!("severe underrun: {:?}", gap);
        }
    }
    self.last_pos = actual;
    self.last_wall_clock = Instant::now();
}
```

用 `Instant::elapsed()` 测量真实流逝时间，而非假定 sleep 精确等于 500ms。消除测不准的自身误差。

---

## 6. Tests

### 6.1 wav_roundtrip (离线, CI 可跑)

```
cargo test wav_roundtrip -- --nocapture
```

- 生成参考信号 → rodio Decoder 解码 → 逐样本对比
- 断言: 样本数相等 + max_diff < 0.01 + silence_ratio < 1% + NaN/Inf = 0

### 6.2 diagnose_cached_audio_quality (离线, 增强版)

```
cargo test diagnose_cached_audio_quality -- --nocapture
```

现有测试增强：
- 解码后额外调用 `silence_gaps()` / `amplitude_jumps()` / `harmonic_snr()`
- println 输出指标，不改变现有断言
- 依赖 `/tmp/music_cache/` 中有缓存文件（需先手动播放一次）

### 6.3 在线探针（手动）

前端 console 监听 `play_probe_report` Tauri event，输出 `ProbeReport` JSON。开发者手动播放歌曲后查看。

---

## 7. 数据驱动决策

第一期跑完后产出基线数据：

| 测试 | WSL2 预期 | Windows 原生预期 | macOS 预期 |
|---|---|---|---|
| wav_roundtrip | 全通过 | 全通过 | 全通过 |
| silence_gaps | 0 | 0 | 0 |
| amplitude_jumps | 0 | 0 | 0 |
| harmonic_snr | > 25dB | > 25dB | > 25dB |
| probe.stall_count | 可能有值 | 0 | 0 |

| 结果 | 含义 | 动作 |
|---|---|---|
| 离线全过 + 在线 stall=0 | 颗粒感是 WSL2 OS 层问题 | 不修应用代码 |
| 离线全过 + 在线 stall>0 | 输出层 buffer 不足 | 推方案 B (cpal 直出) |
| 离线 wav_roundtrip 失败 | Symphonia 解码有问题 | 排查格式/版本 |
| 离线 silence_gaps > 0 | 解码器产生静音输出 | 排查特定格式兼容性 |

---

## 8. Non-goals

- 不修改 `play()` / `parse_track_request()` 函数签名
- 不引入新 Cargo 依赖
- 不改变 `MusicHandler` 公开 API
- 不在前端增加新 UI（探针报告走 console/Tauri event）
