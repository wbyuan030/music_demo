use std::collections::HashMap;
use std::sync::Arc;

use reqwest::Client;
use tokio::sync::Mutex;

/// Options passed to every extractor at runtime.
#[derive(Debug, Clone)]
pub struct ExtractorOptions {
    pub user_agent: String,
    pub max_redirect_depth: u32,
    pub language: String,
    pub region: String,
    pub proxy: Option<String>,
    /// 端点覆盖（测试用）：name → base URL。
    /// extractor 应通过 `options.endpoint("name", default_url)` 取端点，
    /// 测试用 `with_endpoint("search", "http://127.0.0.1:port")` 注入 mock。
    pub endpoints: HashMap<String, String>,
}

impl ExtractorOptions {
    /// 取命名端点：优先注入值，否则回退 default_url。
    pub fn endpoint(&self, name: &str, default_url: &str) -> String {
        self.endpoints
            .get(name)
            .cloned()
            .unwrap_or_else(|| default_url.to_string())
    }

    /// 测试/配置用：覆盖命名端点。
    pub fn with_endpoint(mut self, name: &str, url: &str) -> Self {
        self.endpoints.insert(name.to_string(), url.to_string());
        self
    }
}

impl Default for ExtractorOptions {
    fn default() -> Self {
        Self {
            user_agent: concat!(
                "Mozilla/5.0 (X11; Linux x86_64) ",
                "AppleWebKit/537.36 (KHTML, like Gecko) ",
                "Chrome/120.0.0.0 Safari/537.36"
            )
            .to_string(),
            max_redirect_depth: 5,
            language: "en".to_string(),
            region: "US".to_string(),
            proxy: None,
            endpoints: HashMap::new(),
        }
    }
}

/// Shared runtime context for all extractors.
///
/// Every extractor MUST use this context for HTTP, caching, etc.
/// Extractors MUST NOT create their own HTTP clients.
#[derive(Clone)]
pub struct ExtractorContext {
    pub http: Arc<Client>,
    pub options: ExtractorOptions,
    pub cancellation: CancellationToken,
    pub logger: Arc<dyn ExtractLogger + Send + Sync>,
}

impl ExtractorContext {
    /// Create a new context with a default reqwest Client.
    pub fn new() -> Result<Self, anyhow::Error> {
        let client = Client::builder()
            .gzip(true)
            .cookie_store(true)
            .user_agent(&ExtractorOptions::default().user_agent)
            .build()?;

        Ok(Self {
            http: Arc::new(client),
            options: ExtractorOptions::default(),
            cancellation: CancellationToken::new(),
            logger: Arc::new(NoopLogger),
        })
    }

    /// Create with a custom reqwest Client.
    pub fn with_client(client: Client) -> Self {
        Self {
            http: Arc::new(client),
            options: ExtractorOptions::default(),
            cancellation: CancellationToken::new(),
            logger: Arc::new(NoopLogger),
        }
    }

    /// Check if the operation has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    /// Builder-style: set options.
    pub fn with_options(mut self, opts: ExtractorOptions) -> Self {
        self.options = opts;
        self
    }

    /// Builder-style: set logger.
    pub fn with_logger(mut self, logger: Arc<dyn ExtractLogger + Send + Sync>) -> Self {
        self.logger = logger;
        self
    }
}

/// Simple cancellation token.
#[derive(Clone)]
pub struct CancellationToken {
    cancelled: Arc<Mutex<bool>>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(Mutex::new(false)),
        }
    }
    pub fn cancel(&self) {
        let mut c = self.cancelled.try_lock();
        if let Ok(ref mut c) = c {
            **c = true;
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.try_lock().map(|c| *c).unwrap_or(false)
    }
}

/// Abstract logger for extractors.
pub trait ExtractLogger: Send + Sync {
    fn info(&self, msg: &str);
    fn warn(&self, msg: &str);
    fn error(&self, msg: &str);
    fn debug(&self, msg: &str);
}

struct NoopLogger;

impl ExtractLogger for NoopLogger {
    fn info(&self, _msg: &str) {}
    fn warn(&self, _msg: &str) {}
    fn error(&self, _msg: &str) {}
    fn debug(&self, _msg: &str) {}
}

/// A logger that forwards to the `log` crate.
pub struct LogLogger;

impl ExtractLogger for LogLogger {
    fn info(&self, msg: &str) {
        log::info!("[extractor] {}", msg);
    }
    fn warn(&self, msg: &str) {
        log::warn!("[extractor] {}", msg);
    }
    fn error(&self, msg: &str) {
        log::error!("[extractor] {}", msg);
    }
    fn debug(&self, msg: &str) {
        log::debug!("[extractor] {}", msg);
    }
}
