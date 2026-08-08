use serde::{Deserialize, Serialize};

// ── InnerTube API request types ───────────────────────────────────────

/// InnerTube API request context (embedded in every request).
#[derive(Debug, Serialize)]
pub struct InnerTubeContext {
    pub client: ClientContext,
}

#[derive(Debug, Serialize)]
pub struct ClientContext {
    #[serde(rename = "clientName")]
    pub client_name: String,
    #[serde(rename = "clientVersion")]
    pub client_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub android_sdk_version: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
}

/// Search request body.
#[derive(Debug, Serialize)]
pub struct SearchRequest {
    pub context: InnerTubeContext,
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<String>,
}

/// Player request body.
#[derive(Debug, Serialize)]
pub struct PlayerRequest {
    pub context: InnerTubeContext,
    #[serde(rename = "videoId")]
    pub video_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "playbackContext")]
    pub playback_context: Option<PlaybackContext>,
}

#[derive(Debug, Serialize)]
pub struct PlaybackContext {
    pub content_player_playback_context: ContentPlayerPlaybackContext,
}

#[derive(Debug, Serialize)]
pub struct ContentPlayerPlaybackContext {
    #[serde(rename = "signatureTimestamp")]
    pub signature_timestamp: i64,
}

// ── InnerTube API response types ──────────────────────────────────────

/// Top-level InnerTube API response.
#[derive(Debug, Deserialize)]
pub struct InnerTubeResponse {
    pub contents: Option<Contents>,
    #[serde(rename = "continuationContents")]
    pub continuation_contents: Option<serde_json::Value>,
    #[serde(rename = "streamingData")]
    pub streaming_data: Option<StreamingData>,
    #[serde(rename = "videoDetails")]
    pub video_details: Option<VideoDetails>,
    pub error: Option<ApiError>,
    pub playability_status: Option<PlayabilityStatus>,
}

#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub code: i64,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct PlayabilityStatus {
    pub status: String,
    pub reason: Option<String>,
}

/// Search result contents.
#[derive(Debug, Deserialize)]
pub struct Contents {
    #[serde(rename = "tabbedSearchResultsRenderer")]
    pub tabbed_search: Option<TabbedSearchResultsRenderer>,
    #[serde(rename = "twoColumnSearchResultsRenderer")]
    pub two_column_search: Option<TwoColumnSearchResultsRenderer>,
}

/// Tabbed search (YouTube Music style).
#[derive(Debug, Deserialize)]
pub struct TabbedSearchResultsRenderer {
    pub tabs: Vec<Tab>,
}

#[derive(Debug, Deserialize)]
pub struct Tab {
    #[serde(rename = "tabRenderer")]
    pub tab_renderer: Option<TabRenderer>,
}

#[derive(Debug, Deserialize)]
pub struct TabRenderer {
    pub content: Option<TabContent>,
}

#[derive(Debug, Deserialize)]
pub struct TabContent {
    #[serde(rename = "sectionListRenderer")]
    pub section_list: Option<SectionListRenderer>,
}

/// Standard two-column search (YouTube main site).
#[derive(Debug, Deserialize)]
pub struct TwoColumnSearchResultsRenderer {
    #[serde(rename = "primaryContents")]
    pub primary_contents: Option<PrimaryContents>,
}

#[derive(Debug, Deserialize)]
pub struct PrimaryContents {
    #[serde(rename = "sectionListRenderer")]
    pub section_list: Option<SectionListRenderer>,
}

#[derive(Debug, Deserialize)]
pub struct SectionListRenderer {
    pub contents: Vec<SectionContent>,
}

#[derive(Debug, Deserialize)]
pub struct SectionContent {
    #[serde(rename = "musicShelfRenderer")]
    pub music_shelf: Option<MusicShelfRenderer>,
    #[serde(rename = "itemSectionRenderer")]
    pub item_section: Option<ItemSectionRenderer>,
    #[serde(rename = "continuationItemRenderer")]
    pub continuation_item: Option<ContinuationItemRenderer>,
}

/// Music shelf (holds search results on YouTube Music).
#[derive(Debug, Deserialize)]
pub struct MusicShelfRenderer {
    pub contents: Vec<MusicShelfItem>,
    pub continuations: Option<Vec<ContinuationItem>>,
}

#[derive(Debug, Deserialize)]
pub struct MusicShelfItem {
    #[serde(rename = "musicResponsiveListItemRenderer")]
    pub renderer: Option<MusicResponsiveListItemRenderer>,
}

#[derive(Debug, Deserialize)]
pub struct ItemSectionRenderer {
    pub contents: Vec<ItemSectionContent>,
}

#[derive(Debug, Deserialize)]
pub struct ItemSectionContent {
    #[serde(rename = "musicResponsiveListItemRenderer")]
    pub music_renderer: Option<MusicResponsiveListItemRenderer>,
    #[serde(rename = "videoRenderer")]
    pub video_renderer: Option<VideoRenderer>,
    #[serde(rename = "playlistRenderer")]
    pub playlist_renderer: Option<serde_json::Value>,
    #[serde(rename = "channelRenderer")]
    pub channel_renderer: Option<serde_json::Value>,
    #[serde(rename = "continuationItemRenderer")]
    pub continuation_item: Option<ContinuationItemRenderer>,
}

// ── Music Responsive List Item (YouTube Music search result) ──────────

#[derive(Debug, Deserialize)]
pub struct MusicResponsiveListItemRenderer {
    #[serde(rename = "playlistItemData")]
    pub playlist_item_data: Option<PlaylistItemData>,
    #[serde(rename = "flexColumns")]
    pub flex_columns: Vec<FlexColumn>,
    #[serde(rename = "fixedColumns")]
    pub fixed_columns: Option<Vec<FlexColumn>>,
    pub thumbnail: Option<MusicThumbnail>,
    #[serde(rename = "navigationEndpoint")]
    pub navigation_endpoint: Option<NavigationEndpoint>,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistItemData {
    #[serde(rename = "videoId")]
    pub video_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FlexColumn {
    #[serde(rename = "musicResponsiveListItemFlexColumnRenderer")]
    pub renderer: Option<FlexColumnRenderer>,
}

#[derive(Debug, Deserialize)]
pub struct FlexColumnRenderer {
    pub text: RunsText,
}

#[derive(Debug, Deserialize)]
pub struct RunsText {
    pub runs: Option<Vec<Run>>,
    #[serde(rename = "simpleText")]
    pub simple_text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Run {
    pub text: String,
    #[serde(rename = "navigationEndpoint")]
    pub navigation_endpoint: Option<NavigationEndpoint>,
}

#[derive(Debug, Deserialize)]
pub struct NavigationEndpoint {
    #[serde(rename = "watchEndpoint")]
    pub watch_endpoint: Option<WatchEndpoint>,
    #[serde(rename = "browseEndpoint")]
    pub browse_endpoint: Option<BrowseEndpoint>,
}

#[derive(Debug, Deserialize)]
pub struct WatchEndpoint {
    #[serde(rename = "videoId")]
    pub video_id: Option<String>,
    #[serde(rename = "playlistId")]
    pub playlist_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BrowseEndpoint {
    #[serde(rename = "browseId")]
    pub browse_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MusicThumbnail {
    #[serde(rename = "musicThumbnailRenderer")]
    pub renderer: Option<MusicThumbnailRenderer>,
}

#[derive(Debug, Deserialize)]
pub struct MusicThumbnailRenderer {
    pub thumbnail: ThumbnailData,
}

#[derive(Debug, Deserialize)]
pub struct ThumbnailData {
    pub thumbnails: Vec<Thumbnail>,
}

#[derive(Debug, Deserialize)]
pub struct Thumbnail {
    pub url: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
}

// ── Video Renderer (standard YouTube search result) ───────────────────

#[derive(Debug, Deserialize)]
pub struct VideoRenderer {
    #[serde(rename = "videoId")]
    pub video_id: Option<String>,
    pub title: Option<RunsText>,
    #[serde(rename = "lengthText")]
    pub length_text: Option<RunsText>,
    #[serde(rename = "ownerText")]
    pub owner_text: Option<RunsText>,
    #[serde(rename = "shortBylineText")]
    pub short_byline: Option<RunsText>,
    pub thumbnail: Option<ThumbnailData>,
    #[serde(rename = "publishedTimeText")]
    pub published_time: Option<RunsText>,
    #[serde(rename = "viewCountText")]
    pub view_count: Option<RunsText>,
    #[serde(rename = "lengthSeconds")]
    pub length_seconds: Option<String>,
}

// ── Pagination (continuation) ─────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ContinuationItemRenderer {
    #[serde(rename = "continuationEndpoint")]
    pub continuation_endpoint: Option<ContinuationEndpoint>,
}

#[derive(Debug, Deserialize)]
pub struct ContinuationItem {
    #[serde(rename = "nextContinuationData")]
    pub next_continuation_data: Option<NextContinuationData>,
}

#[derive(Debug, Deserialize)]
pub struct ContinuationEndpoint {
    #[serde(rename = "continuationCommand")]
    pub continuation_command: Option<ContinuationCommand>,
}

#[derive(Debug, Deserialize)]
pub struct ContinuationCommand {
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NextContinuationData {
    pub continuation: String,
}

// ── Streaming data (player response) ──────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct StreamingData {
    #[serde(rename = "expiresInSeconds")]
    pub expires_in_seconds: Option<String>,
    pub formats: Option<Vec<StreamFormat>>,
    #[serde(rename = "adaptiveFormats")]
    pub adaptive_formats: Option<Vec<StreamFormat>>,
    #[serde(rename = "dashManifestUrl")]
    pub dash_manifest_url: Option<String>,
    #[serde(rename = "hlsManifestUrl")]
    pub hls_manifest_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StreamFormat {
    pub itag: Option<i64>,
    pub url: Option<String>,
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    pub bitrate: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    #[serde(rename = "contentLength")]
    pub content_length: Option<String>,
    #[serde(rename = "lastModified")]
    pub last_modified: Option<String>,
    #[serde(rename = "qualityLabel")]
    pub quality_label: Option<String>,
    #[serde(rename = "audioQuality")]
    pub audio_quality: Option<String>,
    #[serde(rename = "audioChannels")]
    pub audio_channels: Option<i64>,
    #[serde(rename = "approxDurationMs")]
    pub approx_duration_ms: Option<String>,
    #[serde(rename = "sampleRate")]
    pub sample_rate: Option<String>,
    #[serde(rename = "averageBitrate")]
    pub average_bitrate: Option<i64>,
    #[serde(rename = "audioTrack")]
    pub audio_track: Option<AudioTrack>,
    #[serde(rename = "highReplication")]
    pub high_replication: Option<bool>,
    #[serde(rename = "licenseInfo")]
    pub license_info: Option<serde_json::Value>,
    /// Cipher (needs to be decoded to get URL).
    #[serde(default)]
    pub cipher: Option<String>,
    #[serde(rename = "signatureCipher")]
    pub signature_cipher: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AudioTrack {
    #[serde(rename = "audioIsDefault")]
    pub audio_is_default: Option<bool>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VideoDetails {
    #[serde(rename = "videoId")]
    pub video_id: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "lengthSeconds")]
    pub length_seconds: Option<String>,
    pub author: Option<String>,
    #[serde(rename = "channelId")]
    pub channel_id: Option<String>,
    #[serde(rename = "shortDescription")]
    pub short_description: Option<String>,
    pub thumbnail: Option<VideoThumbnail>,
    #[serde(rename = "viewCount")]
    pub view_count: Option<String>,
    #[serde(rename = "averageRating")]
    pub average_rating: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct VideoThumbnail {
    pub thumbnails: Vec<Thumbnail>,
}

// ── Helper: extract text from RunsText ────────────────────────────────

impl RunsText {
    pub fn flatten(&self) -> String {
        if let Some(simple) = &self.simple_text {
            return simple.clone();
        }
        if let Some(runs) = &self.runs {
            return runs
                .iter()
                .map(|r| r.text.as_str())
                .collect::<Vec<_>>()
                .join("");
        }
        String::new()
    }
}
