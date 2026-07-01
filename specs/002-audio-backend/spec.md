# Problem Specification: 音频输出颗粒感

**Created**: 2026-06-30

**Status**: Investigating

**Context**: 用户反馈播放音频时有"颗粒感"（crackling / stuttering / grainy texture），类似缓冲欠载产生的破音或断续。

---

## 1. 现象描述

| 属性 | 观察 |
|---|---|
| 触发条件 | 任意音频源（微信/B站），任意时长 |
| 听觉特征 | 间歇性杂音/破音，非连续性爆音 |
| 出现频率 | 持续存在，非偶发 |
| 平台 | WSL2 (Linux 6.18) → Windows 音频桥 |

---

## 2. 已排除的根因

### 2.1 源素材质量 ❌ 已排除

诊断测试 `diagnose_cached_audio_quality` 扫描 `/tmp/music_cache/` 中 3 个缓存文件：

```
📁 fae04d40...bin | 44100 Hz | 2ch | 260.8s
📁 b1c5ca1d...bin | 44100 Hz | 2ch | 119.9s
📁 b3646e76...bin | 44100 Hz | 2ch | 246.2s
```

全部为 CD 音质（44100 Hz 立体声），Symphonia 解码成功且无异常。

### 2.2 rodio 解码侧缓冲 ❌ 已排除

两次缓冲策略升级均未改善：

| 尝试 | 方案 | 结果 |
|---|---|---|
| v1 | `source.buffered()` — 解码器与 Sink 之间加环形缓冲 | 颗粒感仍在 |
| v2 | `decoder.collect()` → `SamplesBuffer` — 全量预解码到内存 | 颗粒感仍在 |

v2 将整个音频预先解码为 `Vec<f32>`，Sink 从纯内存 buffer 拉取 sample，**解码侧已零抖动**。

---

## 3. 当前锁定范围

```
音频字节 (内存)
  → Symphonia 解码 ✅ 正常
    → SamplesBuffer (内存) ✅ 正常
      → rodio Sink ✅ 正常
        → rodio Mixer → OutputStream → cpal → [问题区域]
                                               ↓
                                         PulseAudio / ALSA
                                               ↓
                                         WSL2 音频桥
                                               ↓
                                         Windows 音频设备
```

问题在 **cpal → OS 音频栈** 之间。rodio 0.21 未暴露 cpal 的缓冲区配置接口，`OutputStreamBuilder::open_default_stream()` 使用系统默认参数——在 WSL2 虚拟音频设备上，默认缓冲可能极小（256–512 samples，约 5–11ms），任何调度抖动都会导致欠载。

---

## 4. 诊断测试

已有测试位于 `src-tauri/src/music_handler/utils.rs`：

```
cargo test diagnose_cached_audio_quality -- --nocapture
```

该测试扫描缓存目录中的音频文件，用 rodio `Decoder` 解码并报告采样率/声道/时长。测试通过证明解码链路正常。

### 缺失的测试

当前无测试覆盖 cpal 输出层。原因：
- cpal 需要真实音频设备才能创建 `OutputStream`
- 在无音频设备的 CI/headless 环境中会失败
- rodio 未暴露输出层的可量化指标（如 underrun 计数、实际缓冲大小）

如需覆盖，方案方向：
- 引入 `cpal` 直接依赖，绕过 rodio 构造自定义输出流，暴露缓冲配置
- 或 mock cpal 层做契约测试

---

## 5. 可重现步骤

1. 启动应用 → 搜索或解析任意音频（微信/B站）
2. 点击播放
3. 对比：将同一缓存文件（`/tmp/music_cache/<id>.bin`）用 `ffplay` 播放
   ```bash
   ffplay /tmp/music_cache/*.bin
   ```
4. 若 ffplay 无颗粒感 → 确认问题在应用侧（rodio/cpal）
5. 若 ffplay 也有颗粒感 → 问题在 WSL2 音频桥（OS 层）

---

## 6. 已知约束

- rodio 0.21 的 `OutputStreamBuilder` 未公开缓冲大小配置
- `cpal` 的 `StreamConfig` 可设 `buffer_size: BufferSize::Fixed(n)`，但 rodio 内部自行管理
- WSL2 的音频架构为 PulseAudio → Windows Audio，存在已知的高延迟/抖动问题
- 项目 Cargo.toml 未直接依赖 cpal（仅通过 rodio 间接依赖）

---

## 7. 下一步

1. **隔离测试**：用 `ffplay` 播放同一缓存文件，确认是否为应用侧问题
2. **若确认是应用侧**：研究绕过 rodio 的默认输出，直接使用 cpal 自定义 `OutputStream`，显式设置 `BufferSize::Fixed(1024)` 或更大
3. **若确认是 OS 层**：调优 PulseAudio 配置（`/etc/pulse/daemon.conf` 中的 `default-fragments` / `default-fragment-size-msec`），或切换到 WSLg 音频后端
