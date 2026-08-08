use std::{path::PathBuf, sync::Arc};

use crate::extractor::context::ExtractorContext;

use super::{
    // ==== sync-generated:begin runtime_use ====
    audius::AudiusSource,
    // ==== sync-generated:end runtime_use ====
    bilibili::BilibiliSource,
    catalog::TrackCatalog,
    resolver::ResolverRegistry,
    search::SearchRegistry,
    service::PlaybackService,
    wechat::WechatResolver,
    youtube::YoutubeSource,
};

pub struct BackendRuntime {
    pub context: Arc<ExtractorContext>,
    pub catalog: Arc<TrackCatalog>,
    pub resolvers: Arc<ResolverRegistry>,
    pub search: Arc<SearchRegistry>,
    pub playback: Arc<PlaybackService>,
}

impl BackendRuntime {
    pub fn new(
        context: Arc<ExtractorContext>,
        catalog: Arc<TrackCatalog>,
        cache_dir: PathBuf,
    ) -> Self {
        // 每个来源一个 adapter：同时注册搜索和解析能力。
        let mut registry = ResolverRegistry::new();
        registry.register(YoutubeSource);
        registry.register(BilibiliSource);
        registry.register(WechatResolver);
        // ==== sync-generated:begin runtime_register ====
        registry.register(AudiusSource);
        // ==== sync-generated:end runtime_register ====
        let resolvers = Arc::new(registry);

        let mut search = SearchRegistry::new();
        search.register(YoutubeSource);
        search.register(BilibiliSource);
        // ==== sync-generated:begin runtime_register_search ====
        search.register(AudiusSource);
        // ==== sync-generated:end runtime_register_search ====
        let search = Arc::new(search);
        let playback = Arc::new(PlaybackService::new(
            Arc::clone(&context),
            Arc::clone(&catalog),
            Arc::clone(&resolvers),
            cache_dir,
        ));
        Self {
            context,
            catalog,
            resolvers,
            search,
            playback,
        }
    }
}
