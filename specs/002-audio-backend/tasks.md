# Tasks: 音频质量量化测试体系

**Input**: `specs/002-audio-backend/plan.md`, `specs/002-audio-backend/design.md`

---

## Phase 1: Setup

- [x] T001 Create `src-tauri/src/audio_quality/` directory and `mod.rs`

---

## Phase 2: signal.rs — 参考信号生成

- [x] T002 Implement `reference_sine_440hz_1s()` in `signal.rs`

---

## Phase 3: metrics.rs — 样本级指标

- [x] T003 Implement `silence_gaps()` — 滑动窗口(100ms) RMS < 0.001 标记为 gap
- [x] T004 Implement `amplitude_jumps()` — 连续 sample 差值 > 前窗口最大振幅 × 10
- [x] T005 Implement `harmonic_snr()` — Goertzel 算法检测 440/880/1320/1760Hz 频点能量

---

## Phase 4: probe.rs — 在线播放探针

- [x] T006 Implement `PlaybackProbe` struct + `new()` / `tick()` / `report()`

---

## Phase 5: 测试

- [x] T007 Write `wav_roundtrip` test in `audio_quality/mod.rs`

- [x] T008 Enhance `diagnose_cached_audio_quality` in `utils.rs`

---

## Phase 6: 探针集成

- [x] T009 Integrate `PlaybackProbe` into `handler.rs::spawn_progress`

---

## Dependencies

```
T001 (mod.rs + lib.rs)
  ├─→ T002 (signal.rs)
  ├─→ T003 (metrics: silence_gaps)
  ├─→ T004 (metrics: amplitude_jumps)
  └─→ T005 (metrics: harmonic_snr)

T002 + T003 + T004 + T005
  ├─→ T007 (wav_roundtrip test)
  └─→ T008 (enhance diagnose test)

T006 (probe.rs) → T009 (handler integration)
```

**Parallel**: T002, T003, T004, T005, T006 无相互依赖，可并行。
