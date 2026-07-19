use app_lib::extractor::context::ExtractorContext;
use app_lib::extractor::youtube;


use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "music-cli", about = "Music extractor CLI for testing")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Search YouTube Music for tracks
    Search {
        /// Search query
        query: String,
        /// Result section: songs, albums, videos, artists
        #[arg(long, default_value = "songs")]
        section: String,
        /// Output format: table, json
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// Get playable audio manifest for a YouTube video
    Manifest {
        /// YouTube video ID or URL
        video: String,
    },
    /// Show video info (title, duration, etc.)
    Info {
        /// YouTube video ID or URL
        video: String,
    },
    /// Extract a URL and show the result tree
    Extract {
        /// URL to extract
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
        Command::Search { query, section, format } => {
            search(&ctx, &query, &section, &format).await;
        }
        Command::Manifest { video } => {
            manifest(&ctx, &video).await;
        }
        Command::Info { video } => {
            info(&ctx, &video).await;
        }
        Command::Extract { url } => {
            extract(&ctx, &url).await;
        }
    }
}

async fn search(ctx: &ExtractorContext, query: &str, section: &str, format: &str) {
    match youtube::search::search_music(ctx, query, Some(section)).await {
        Ok(tracks) => {
            if tracks.is_empty() {
                println!("No results found.");
                return;
            }

            match format {
                "json" => {
                    let json = serde_json::to_string_pretty(&tracks).unwrap_or_default();
                    println!("{}", json);
                }
                _ => {
                    println!("Found {} track(s):\n", tracks.len());
                    for (i, track) in tracks.iter().enumerate() {
                        let artist = track.artists.join(", ");
                        let duration = track
                            .duration_ms
                            .map(|ms| format_duration(ms))
                            .unwrap_or_else(|| "?".to_string());
                        let cover = track
                            .artwork
                            .first()
                            .map(|img| &img.url[..img.url.len().min(60)])
                            .unwrap_or("(none)");
                        let vid = track.id.strip_prefix("yt:").unwrap_or(&track.id);

                        println!(
                            "  {}. {:>2}m {} — {} [{}]",
                            i + 1,
                            duration,
                            track.title,
                            artist,
                            vid
                        );
                        if !track.artwork.is_empty() {
                            println!("     cover: {}", cover);
                        }
                        if let Some(album) = &track.album {
                            println!("     album: {}", album);
                        }
                        println!();
                    }

                    // Summary
                    println!("─── {} track(s) ───", tracks.len());
                }
            }
        }
        Err(e) => {
            eprintln!("Search failed: {}", e);
            std::process::exit(1);
        }
    }
}

async fn manifest(ctx: &ExtractorContext, video: &str) {
    let video_id = youtube::extract_video_id(video)
        .unwrap_or_else(|| video.to_string());

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

            // Validate URL accessibility
            if let Some(best) = manifest.streams.first() {
                match youtube::player::validate_url(ctx, &best.url).await {
                    Ok(true) => println!("\n✓ Audio URL is accessible (HEAD request OK)"),
                    Ok(false) => println!("\n✗ Audio URL returned an error status"),
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

async fn info(ctx: &ExtractorContext, video: &str) {
    let video_id = youtube::extract_video_id(video)
        .unwrap_or_else(|| video.to_string());

    println!("Fetching info for video: {}\n", video_id);

    match youtube::player::get_manifest(ctx, &video_id).await {
        Ok(manifest) => {
            println!("Manifest retrieved successfully");
            println!("  Available streams: {}", manifest.streams.len());

            let audio = &manifest.streams;
            if let Some(best) = audio.first() {
                println!("  Best audio: {} @ {} kbps",
                    best.mime_type,
                    best.bitrate.map(|b| (b / 1000) as u64).unwrap_or(0),
                );
                println!("  Content length: {}",
                    best.content_length.map(|l| l.to_string()).unwrap_or_else(|| "unknown".to_string()),
                );
            }

            // Show audio format details
            if audio.len() > 1 {
                println!("\nAll audio formats:");
                for (i, s) in audio.iter().enumerate() {
                    println!("  {}. {} ({} kbps)",
                        i + 1,
                        s.mime_type,
                        s.bitrate.map(|b| (b / 1000) as u64).unwrap_or(0),
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("Info fetch failed: {}", e);
            std::process::exit(1);
        }
    }
}

async fn extract(ctx: &ExtractorContext, url: &str) {
    use app_lib::extractor::protocol::{ExtractInput, Extractor};

    let extractor = youtube::YouTubeMusicExtractor;

    if !extractor.matches(&ExtractInput::new(url)) {
        eprintln!("No matching extractor for URL: {}", url);
        std::process::exit(1);
    }

    println!("Extracting: {}\n", url);

    match extractor.extract(ExtractInput::new(url), ctx).await {
        Ok(result) => match result {
            app_lib::extractor::protocol::ExtractorResult::Media(info) => {
                println!("→ Media");
                println!("  id:    {}", info.id.unwrap_or_default());
                println!("  title: {}", info.title.unwrap_or_default());
                println!("  formats: {}", info.formats.len());
                if !info.extra.is_empty() {
                    println!("  extra keys: {:?}", info.extra.keys().collect::<Vec<_>>());
                }
            }
            app_lib::extractor::protocol::ExtractorResult::Playlist(info) => {
                println!("→ Playlist");
                println!("  title:   {}", info.title.unwrap_or_default());
                println!("  entries: {}", info.entries.len());
                for (i, entry) in info.entries.iter().enumerate() {
                    if let app_lib::extractor::protocol::ExtractorResult::Media(m) = entry {
                        println!("  {}. {} [{}]", i + 1, m.title.as_deref().unwrap_or("?"), m.id.as_deref().unwrap_or("?"));
                    }
                }
            }
            app_lib::extractor::protocol::ExtractorResult::Redirect(info) => {
                println!("→ Redirect");
                println!("  url:    {}", info.url);
                println!("  ie_key: {:?}", info.ie_key);
            }
            app_lib::extractor::protocol::ExtractorResult::TransparentRedirect(info) => {
                println!("→ TransparentRedirect");
                println!("  url:  {}", info.url);
                println!("  ie_key: {:?}", info.ie_key);
            }
            app_lib::extractor::protocol::ExtractorResult::MultiMedia(info) => {
                println!("→ MultiMedia");
                println!("  title:   {}", info.title.unwrap_or_default());
                println!("  entries: {}", info.entries.len());
            }
        },
        Err(e) => {
            eprintln!("Extraction failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn format_duration(ms: u64) -> String {
    let total_secs = ms / 1000;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{}:{:02}", mins, secs)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}
