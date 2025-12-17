//! Reddit wallpaper downloader
//!
//! Downloads wallpapers from Reddit subreddits like r/wallpapers, r/earthporn, etc.
//! Uses Reddit's JSON API (append .json to any subreddit URL).

use super::DownloadOptions;
use super::client::WallflowClient as Client;
use super::filesystem::FilesystemHelper;
use super::traits::{Wallpaper, WallpaperDownloader};
use crate::config::Config;
use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use chrono::Utc;
use rand::seq::SliceRandom;
use serde::Deserialize;
use std::path::Path;
use tracing::debug;

/// Default subreddit if none specified
const DEFAULT_SUBREDDIT: &str = "wallpapers";

/// Reddit API response structure
#[derive(Debug, Deserialize)]
struct RedditResponse {
  data: RedditData,
}

#[derive(Debug, Deserialize)]
struct RedditData {
  children: Vec<RedditChild>,
}

#[derive(Debug, Deserialize)]
struct RedditChild {
  data: RedditPost,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RedditPost {
  /// Direct URL to the image
  url: String,
  /// Post permalink for source attribution
  permalink: String,
  /// Title of the post
  #[allow(dead_code)]
  title: String,
  /// Whether the post is marked as NSFW
  over_18: bool,
}

/// Reddit wallpaper downloader
pub struct RedditDownloader;

impl RedditDownloader {
  /// Check if URL points to a supported image format
  fn is_image_url(url: &str) -> bool {
    let lower = url.to_lowercase();
    lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".png") || lower.ends_with(".webp")
  }

  /// Convert imgur.com URLs to direct image URLs
  fn normalize_imgur_url(url: &str) -> String {
    // Convert imgur.com/abc123 to i.imgur.com/abc123.jpg
    if url.starts_with("http://imgur.com/") || url.starts_with("https://imgur.com/") {
      let normalized = url.replace("://imgur.com/", "://i.imgur.com/");
      if !Self::is_image_url(&normalized) {
        return format!("{}.jpg", normalized);
      }
      return normalized;
    }
    url.to_string()
  }
}

#[async_trait]
impl WallpaperDownloader for RedditDownloader {
  /// Download a wallpaper from Reddit
  /// Query parameters specify subreddit(s) (e.g., "wallflow reddit earthporn" or "wallflow reddit wallpapers+earthporn")
  async fn download(&self, config: &Config, query: &[String], opts: &DownloadOptions) -> Result<Wallpaper> {
    // Use first query param as subreddit, or default
    let subreddit = query.first().map(|s| s.as_str()).unwrap_or(DEFAULT_SUBREDDIT);

    // Build Reddit JSON URL
    let reddit_url = format!("https://www.reddit.com/r/{}.json?limit=100", subreddit);
    debug!("Fetching from Reddit: {}", reddit_url);

    let client = Client::from(&config.advanced);
    let response = client.get(&reddit_url).send().await.context("Failed to send request to Reddit")?;

    if !response.status().is_success() {
      let status = response.status();
      let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
      return Err(anyhow!("Reddit API request failed with status {}: {}", status, error_text));
    }

    let reddit_data: RedditResponse = response.json().await.context("Failed to parse Reddit API response")?;

    if reddit_data.data.children.is_empty() {
      return Err(anyhow!("No posts found in r/{}", subreddit));
    }

    // Filter to image posts only, excluding NSFW
    let image_posts: Vec<&RedditPost> = reddit_data
      .data
      .children
      .iter()
      .map(|child| &child.data)
      .filter(|post| {
        let url = Self::normalize_imgur_url(&post.url);
        Self::is_image_url(&url) && !post.over_18
      })
      .collect();

    if image_posts.is_empty() {
      return Err(anyhow!("No suitable images found in r/{}", subreddit));
    }

    // Pick a random image
    let post = image_posts
      .choose(&mut rand::thread_rng())
      .ok_or_else(|| anyhow!("Failed to select random Reddit image"))?;

    let image_url = Self::normalize_imgur_url(&post.url);
    debug!("Selected Reddit image: {}", image_url);

    // Download the image
    let image_response = client.get(&image_url).send().await.context("Failed to download Reddit image")?;

    if !image_response.status().is_success() {
      return Err(anyhow!("Reddit image download failed with status: {}", image_response.status()));
    }

    let bytes = image_response.bytes().await.context("Failed to read Reddit image data")?;

    // Extract file extension from URL
    let file_extension = image_url
      .rsplit('.')
      .next()
      .and_then(|ext| {
        let ext = ext.split('?').next().unwrap_or(ext);
        if ext.len() <= 5 { Some(ext) } else { None }
      })
      .unwrap_or("jpg");

    let filename = format!("{}_{}", self.source_name(), FilesystemHelper::make_file_suffix());
    let download_dir = opts.output_dir.as_deref().unwrap_or(Path::new(&config.paths.downloads));
    let file_path = download_dir.join(&filename).with_extension(file_extension);

    // Ensure download directory exists
    if let Some(parent) = file_path.parent() {
      tokio::fs::create_dir_all(parent).await.context("Failed to create download directory")?;
    }

    tokio::fs::write(&file_path, &bytes).await.context("Failed to save Reddit image")?;

    let wallpaper = Wallpaper {
      file_path,
      downloaded_at: Utc::now(),
      source: self.source_name().to_string(),
    };

    debug!("Downloaded Reddit wallpaper: {:?}", wallpaper);
    Ok(wallpaper)
  }

  fn source_name(&self) -> &'static str {
    "reddit"
  }
}
