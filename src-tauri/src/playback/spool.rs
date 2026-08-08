use std::{
    io::{self, Read, Seek, SeekFrom},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
};

use parking_lot::{Condvar, Mutex};

/// 边下边播的共享状态：下载器写，解码器读。
///
/// - `downloaded` / `total`：已写入字节数与总长度（未知则为 None）
/// - `done`：下载结束（成功或失败）；`failed`：失败原因
/// - `decoded`：解码器成功打开后置位，下载器等它才提交缓存
pub struct SpoolState {
    pub downloaded: AtomicU64,
    pub total: Mutex<Option<u64>>,
    done: AtomicBool,
    failed: Mutex<Option<String>>,
    decoded_ok: AtomicBool,
    decode_failed: AtomicBool,
    decoded_notify: tokio::sync::Notify,
    /// 阻塞读等待（解码线程在下载边界 sleep）
    inner: Mutex<()>,
    condvar: Condvar,
}

impl SpoolState {
    pub fn new() -> Self {
        Self {
            downloaded: AtomicU64::new(0),
            total: Mutex::new(None),
            done: AtomicBool::new(false),
            failed: Mutex::new(None),
            decoded_ok: AtomicBool::new(false),
            decode_failed: AtomicBool::new(false),
            decoded_notify: tokio::sync::Notify::new(),
            inner: Mutex::new(()),
            condvar: Condvar::new(),
        }
    }

    pub fn is_done(&self) -> bool {
        self.done.load(Ordering::Acquire)
    }

    pub fn failure(&self) -> Option<String> {
        self.failed.lock().clone()
    }

    /// 新数据到达或终态翻转后唤醒阻塞的读。
    pub fn signal(&self) {
        self.condvar.notify_all();
    }

    pub fn add_downloaded(&self, n: u64) {
        self.downloaded.fetch_add(n, Ordering::AcqRel);
        self.signal();
    }

    /// 下载结束（成功时 failed=None；失败时给出原因）。之后读者不再等待新数据。
    pub fn finish(&self, failed: Option<String>) {
        *self.failed.lock() = failed;
        self.done.store(true, Ordering::Release);
        self.signal();
    }

    /// 解码器成功打开后调用；下载器等它才原子提交缓存文件。
    pub fn mark_decoded(&self) {
        self.decoded_ok.store(true, Ordering::Release);
        self.decoded_notify.notify_one();
    }

    pub fn decoded(&self) -> bool {
        self.decoded_ok.load(Ordering::Acquire)
    }

    /// 解码器确认音频无效后调用；下载器中止提交并清理临时文件。
    pub fn mark_decode_failed(&self) {
        self.decode_failed.store(true, Ordering::Release);
        self.decoded_notify.notify_one();
    }

    pub fn decode_failed(&self) -> bool {
        self.decode_failed.load(Ordering::Acquire)
    }

    /// 等待解码器打开成功，或取消。
    pub async fn wait_decoded(&self, cancel: &tokio_util::sync::CancellationToken) -> bool {
        if self.decoded() {
            return true;
        }
        tokio::select! {
            _ = self.decoded_notify.notified() => true,
            _ = cancel.cancelled() => false,
        }
    }

    /// 阻塞等待直到 `pos` 之前的数据可用、或下载结束。
    fn wait_readable(&self, pos: u64) {
        let mut guard = self.inner.lock();
        while !self.done.load(Ordering::Acquire) && pos >= self.downloaded.load(Ordering::Acquire) {
            self.condvar.wait(&mut guard);
        }
    }
}

impl Default for SpoolState {
    fn default() -> Self {
        Self::new()
    }
}

/// `Read + Seek + Send + Sync` 的组合 trait，用于统一完整文件与 spool 两种解码输入。
pub trait ReadSeek: Read + Seek + Send + Sync {}
impl<T: Read + Seek + Send + Sync + ?Sized> ReadSeek for T {}

/// 提供给 rodio（`Read + Seek`）的阻塞读句柄。
/// 读越过下载边界时阻塞等数据；下载失败时返回错误；正常结束返回 EOF。
pub struct BlockingSpoolReader {
    file: std::fs::File,
    pos: u64,
    state: Arc<SpoolState>,
}

impl BlockingSpoolReader {
    pub fn new(file: std::fs::File, state: Arc<SpoolState>) -> Self {
        Self {
            file,
            pos: 0,
            state,
        }
    }
}

impl Read for BlockingSpoolReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let downloaded = self.state.downloaded.load(Ordering::Acquire);
            if self.pos < downloaded {
                let want = buf.len().min((downloaded - self.pos) as usize);
                // 文件句柄为 reader 独占，按自维护的 pos 定位后读取。
                (&self.file).seek(SeekFrom::Start(self.pos))?;
                let n = (&self.file).read(&mut buf[..want])?;
                if n == 0 {
                    // 写端在同一进程，正常不应出现；等待写端刷新。
                    self.state.wait_readable(self.pos);
                    continue;
                }
                self.pos += n as u64;
                return Ok(n);
            }
            if self.state.is_done() {
                if let Some(error) = self.state.failure() {
                    return Err(io::Error::other(error));
                }
                return Ok(0);
            }
            self.state.wait_readable(self.pos);
        }
    }
}

impl Seek for BlockingSpoolReader {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let new_pos = match from {
            SeekFrom::Start(offset) => offset,
            SeekFrom::Current(delta) => {
                let base = self.pos as i128;
                let target = base + delta as i128;
                if target < 0 {
                    return Err(io::Error::other("seek before start of stream"));
                }
                target as u64
            }
            SeekFrom::End(offset) => {
                let total = match *self.state.total.lock() {
                    Some(total) => total,
                    None => {
                        // 未知总长：等下载结束才能知道终点。
                        while !self.state.is_done() {
                            self.state.wait_readable(u64::MAX);
                        }
                        self.state.downloaded.load(Ordering::Acquire)
                    }
                };
                let target = total as i128 + offset as i128;
                if target < 0 {
                    return Err(io::Error::other("seek before start of stream"));
                }
                target as u64
            }
        };
        self.pos = new_pos;
        Ok(new_pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    #[test]
    fn reader_blocks_until_data_then_eof() {
        let state = Arc::new(SpoolState::new());
        let (mut writer, file) = spool_pair(&state);
        let writer_state = state.clone();
        let writer_thread = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(50));
            writer.write_all(b"abcd").unwrap();
            writer_state.add_downloaded(4);
            writer_state.finish(None);
        });

        let mut reader = BlockingSpoolReader::new(file, state.clone());
        let mut buf = [0u8; 8];
        // 数据未到：阻塞等待；到达后读出全部内容。
        assert_eq!(reader.read(&mut buf).unwrap(), 4);
        assert_eq!(&buf[..4], b"abcd");
        // 下载正常结束：EOF
        assert_eq!(reader.read(&mut buf).unwrap(), 0);
        writer_thread.join().unwrap();
    }

    #[test]
    fn reader_reports_failure_after_download_error() {
        let state = Arc::new(SpoolState::new());
        let (_writer, file) = spool_pair(&state);
        let mut reader = BlockingSpoolReader::new(file, state.clone());
        state.finish(Some("network down".to_string()));
        assert!(reader.read(&mut [0u8; 1]).is_err());
    }

    #[test]
    fn seek_works_within_downloaded_region() {
        let state = Arc::new(SpoolState::new());
        let (mut writer, file) = spool_pair(&state);
        writer.write_all(b"0123456789").unwrap();
        state.add_downloaded(10);
        *state.total.lock() = Some(10);
        let mut reader = BlockingSpoolReader::new(file, state.clone());
        reader.seek(SeekFrom::Start(8)).unwrap();
        let mut buf = [0u8; 4];
        assert_eq!(reader.read(&mut buf).unwrap(), 2);
        assert_eq!(&buf[..2], b"89");
    }

    fn spool_pair(_state: &Arc<SpoolState>) -> (std::fs::File, std::fs::File) {
        let path = std::env::temp_dir().join(format!("spool-test-{}.bin", uuid::Uuid::new_v4()));
        let file = std::fs::File::create(&path).unwrap();
        let reader_file = std::fs::File::open(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        (file, reader_file)
    }
}
