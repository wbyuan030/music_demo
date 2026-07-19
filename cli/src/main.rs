use app_lib::extractor::bilibili;
use app_lib::extractor::context::ExtractorContext;
use app_lib::extractor::protocol::{ExtractInput, Extractor, ExtractorResult};
use app_lib::extractor::youtube;

use clap::{Parser, Subcommand, ValueEnum};

/// Search source.
#[derive(ValueEnum, Clone, Default)]
enum Source {
    #[default]
    All,
    Youtube,
    Bilibili,
}

#[derive(Parser)]
#[command(name = "music-cli", about = "Music extractor CLI for testing")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Search for tracks across sources
    Search {
        query: String,
        /// Source: youtube, bilibili, all
        #[arg(long, default_value = "all")]
        source: Source,
        /// YouTube section: songs, albums, videos, artists
        #[arg(long, default_value = "songs")]
        section: String,
        /// Output format: table, json
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// Get YouTube audio manifest
    Manifest {
        video: String,
    },
    /// Show YouTube video info
    Info {
        video: String,
    },
    /// Get Bilibili audio manifest
    ManifestBili {
        video: String,
    },
    /// Extract a URL using the extractor framework
    Extract {
        url: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let ctx = match ExtractorContext::new() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error creating extractor context: {}", e);
            std::process::exit(1);
        }
    };

    match cli.command {
        Command::Search { query, source, section, format } => {
            search(&ctx, &query, &source, &section, &format).await;
        }
        Command::Manifest { video } => {
            manifest(&ctx, &video).await;
        }
        Command::Info { video } => {
            info(&ctx, &video).await;
        }
        Command::ManifestBili { video } => {
            manifest_bili(&ctx, &video).await;
        }
        Command::Extract { url } => {
            extract(&ctx, &url).await;
        }
    }
}

// ── Search (multi-source) ───────────────────────────────────────────

async fn search(ctx: &ExtractorContext, query: &str, source: &Source, section: &str, format: &str) {
    let mut all_tracks: Vec<app_lib::extractor::model::Track> = Vec::new();

    match source {
        Source::All | Source::Youtube => {
            if let Ok(tracks) = youtube::search::search_music(ctx, query, Some(section)).await {
                all_tracks.extend(tracks);
            }
        }
        _ => {}
    }

    match source {
        Source::All | Source::Bilibili => {
            if let Ok(tracks) = bilibili::search::search_video(ctx, query, 1).await {
                all_tracks.extend(tracks);
            }
        }
        _ => {}
    }

    if all_tracks.is_empty() {
        println!("No results found.");
        return;
    }

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&all_tracks).unwrap_or_default();
            println!("{}", json);
        }
        _ => {
            println!("Found {} track(s):\n", all_tracks.len());
            for (i, track) in all_tracks.iter().enumerate() {
                let artist = track.artists.join(", ");
                let duration = track
                    .duration_ms
                    .map(|ms| format_duration(ms))
                    .unwrap_or_else(|| "?".to_string());
                let prefix = if track.id.starts_with("yt:") { "YT" } else { "Bili" };
                let vid = track.id.split(':').last().unwrap_or(&track.id);

                println!(
                    "  {}. {:>4} {} — {} [{}:{}]",
                    i + 1, duration, track.title, artist, prefix, vid
                );
                if !track.artwork.is_empty() {
                    println!("     cover: {}", truncate(&track.artwork[0].url, 60));
                }
                if let Some(album) = &track.album {
                    println!("     album: {}", album);
                }
                println!();
            }
            println!("─── {} track(s) ───", all_tracks.len());
        }
    }
}

// ── YouTube Manifest ─────────────────────────────────────────────────

async fn manifest(ctx: &ExtractorContext, video: &str) {
    let video_id = youtube::extract_video_id(video).unwrap_or_else(|| video.to_string());
    println!("Fetching manifest for video: {}\n", video_id);

    match youtube::player::get_manifest(ctx, &video_id).await {
        Ok(manifest) => {
            println!("Streams: {}", manifest.streams.len());
            for (i, stream) in manifest.streams.iter().enumerate() {
                println!(
                    "  {}. {} — {} kbps, {} bytes",
                    i + 1,
                    stream.mime_type,
                    stream.bitrate.map(|b| (b / 1000) as u64).unwrap_or(0),
                    stream.content_length.map(|l| l.to_string()).unwrap_or_else(|| "?".to_string()),
                );
                println!("     url: {}", truncate(&stream.url, 80));
            }

            println!("\nHeaders:");
            for (k, v) in &manifest.headers {
                println!("  {}: {}", k, v);
            }

            if let Some(expires) = manifest.expires_at {
                println!("\nExpires: {:?}", expires);
            }

            if let Some(best) = manifest.streams.first() {
                match youtube::player::validate_url(ctx, &best.url).await {
                    Ok(true) => println!("\n✓ Audio URL is accessible"),
                    Ok(false) => println!("\n✗ Audio URL returned an error"),
                    Err(e) => println!("\n! Audio URL validation failed: {}", e),
                }
            }

            println!("\n─── manifest end ───");
        }
        Err(e) => {
            eprintln!("Failed to get manifest: {}", e);
            std::process::exit(1);
        }
    }
}

// ── YouTube Info ──────────────────────────────────────────────────────

async fn info(ctx: &ExtractorContext, video: &str) {
    let video_id = youtube::extract_video_id(video).unwrap_or_else(|| video.to_string());
    println!("Fetching info for video: {}\n", video_id);

    match youtube::player::get_manifest(ctx, &video_id).await {
        Ok(manifest) => {
            println!("Manifest retrieved successfully");
            println!("  Streams: {}", manifest.streams.len());

            if let Some(best) = manifest.streams.first() {
                println!("  Best: {} @ {} kbps ({} bytes)",
                    best.mime_type,
                    best.bitrate.map(|b| (b / 1000) as u64).unwrap_or(0),
                    best.content_length.map(|l| l.to_string()).unwrap_or_else(|| "?".to_string()),
                );
            }

            if manifest.streams.len() > 1 {
                println!("\nAll formats:");
                for (i, s) in manifest.streams.iter().enumerate() {
                    println!("  {}. {} ({} kbps)", i + 1, s.mime_type,
                        s.bitrate.map(|b| (b / 1000) as u64).unwrap_or(0));
                }
            }
        }
        Err(e) => {
            eprintln!("Info fetch failed: {}", e);
            std::process::exit(1);
        }
    }
}

// ── Bilibili Manifest ────────────────────────────────────────────────

async fn manifest_bili(ctx: &ExtractorContext, video: &str) {
    let video_id = bilibili::extract_bili_video_id(video).unwrap_or_else(|| video.to_string());
    println!("Fetching Bilibili manifest for: {}\n", video_id);

    match bilibili::player::get_video_manifest(ctx, &video_id).await {
        Ok(manifest) => {
            println!("Streams: {}", manifest.streams.len());
            for (i, stream) in manifest.streams.iter().enumerate() {
                println!(
                    "  {}. {} — {} kbps, {} bytes",
                    i + 1,
                    stream.mime_type,
                    stream.bitrate.map(|b| (b / 1000) as u64).unwrap_or(0),
                    stream.content_length.map(|l| l.to_string()).unwrap_or_else(|| "?".to_string()),
                );
                println!("     url: {}", truncate(&stream.url, 80));
            }

            if !manifest.headers.is_empty() {
                println!("\nHeaders:");
                for (k, v) in &manifest.headers {
                    println!("  {}: {}", k, v);
                }
            }
            println!("\n─── manifest end ───");
        }
        Err(e) => {
            eprintln!("Failed to get manifest: {}", e);
            std::process::exit(1);
        }
    }
}

// ── Extract ──────────────────────────────────────────────────────────

async fn extract(ctx: &ExtractorContext, url: &str) {
    // Try YouTube
    let yt = youtube::YouTubeMusicExtractor;
    if yt.matches(&ExtractInput::new(url)) {
        println!("Using YouTube Music extractor\n");
        match yt.extract(ExtractInput::new(url), ctx).await {
            Ok(r) => print_extract(r),
            Err(e) => { eprintln!("Extraction failed: {}", e); std::process::exit(1); }
        }
        return;
    }

    // Try Bilibili
    let bili = bilibili::BiliBiliExtractor;
    if bili.matches(&ExtractInput::new(url)) {
        println!("Using Bilibili extractor\n");
        match bili.extract(ExtractInput::new(url), ctx).await {
            Ok(r) => print_extract(r),
            Err(e) => { eprintln!("Extraction failed: {}", e); std::process::exit(1); }
        }
        return;
    }

    eprintln!("No matching extractor for URL: {}", url);
    std::process::exit(1);
}
fn print_extract(result: ExtractorResult) {
    match result {
        ExtractorResult::Media(info) => {
            println!("→ Media\n  id: {}  title: {}\n  formats: {}",
                info.id.unwrap_or_default(), info.title.unwrap_or_default(), info.formats.len());
            if !info.extra.is_empty() {
                println!("  extra: {:?}", info.extra.keys().collect::<Vec<_>>());
            }
        }
        ExtractorResult::Playlist(info) => {
            println!("→ Playlist\n  title: {}  entries: {}", info.title.unwrap_or_default(), info.entries.len());
            for (i, entry) in info.entries.iter().enumerate() {
                if let ExtractorResult::Media(m) = entry {
                    println!("  {}. {} [{}]", i + 1, m.title.as_deref().unwrap_or("?"), m.id.as_deref().unwrap_or("?"));
                }
            }
        }
        ExtractorResult::Redirect(info) => {
            println!("→ Redirect\n  url: {}  ie_key: {:?}", info.url, info.ie_key);
        }
        ExtractorResult::TransparentRedirect(info) => {
            println!("→ TransparentRedirect\n  url: {}  ie_key: {:?}", info.url, info.ie_key);
        }
        ExtractorResult::MultiMedia(info) => {
            println!("→ MultiMedia\n  title: {}  entries: {}", info.title.unwrap_or_default(), info.entries.len());
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────

fn format_duration(ms: u64) -> String {
    let total_secs = ms / 1000;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{}:{:02}", mins, secs)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() }
    else { format!("{}...", &s[..max]) }
}
