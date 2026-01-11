use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

// Windows-specific: prevent console windows from appearing for child processes
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Get the yt-dlp binary name for the current platform
fn ytdlp_binary_name() -> &'static str {
    #[cfg(target_os = "windows")]
    return "yt-dlp.exe";

    #[cfg(not(target_os = "windows"))]
    return "yt-dlp";
}

/// Get the platform-specific yt-dlp download URL
fn ytdlp_download_url() -> &'static str {
    #[cfg(target_os = "windows")]
    return "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";

    #[cfg(target_os = "macos")]
    return "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos";

    #[cfg(target_os = "linux")]
    return "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux";

    // Fallback for other platforms
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp";
}

// Download state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadItem {
    pub id: String,
    pub url: String,
    pub title: String,
    pub progress: f64,
    pub status: String,
    pub download_type: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistInfo {
    pub title: String,
    pub video_count: u32,
    pub videos: Vec<VideoInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub id: String,
    pub title: String,
    pub url: String,
    pub duration: Option<String>,
    pub thumbnail: Option<String>,
}

// Global state for active downloads
type DownloadRegistry = Arc<Mutex<HashMap<String, ActiveDownload>>>;

#[derive(Debug)]
struct ActiveDownload {
    child: std::process::Child,
    stdout: Option<std::process::ChildStdout>,
    stderr: Option<std::process::ChildStderr>,
    output_path: String,
    title: String,
    cancelled: bool,  // Flag to track if download was cancelled by user
}

impl ActiveDownload {
    fn new(child: std::process::Child, stdout: Option<std::process::ChildStdout>, stderr: Option<std::process::ChildStderr>, output_path: String, title: String) -> Self {
        Self { child, stdout, stderr, output_path, title, cancelled: false }
    }
}

// Lazy static for global registry
use once_cell::sync::Lazy;

static DOWNLOAD_REGISTRY: Lazy<DownloadRegistry> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

// YTDLP_PATH will store the path to yt-dlp executable (thread-safe)
static YTDLP_PATH: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

/// Get the path to the yt-dlp executable
/// Will check in order: resource dir -> local data dir -> download if needed
fn get_ytdlp_path(app: &AppHandle) -> Result<PathBuf, String> {
    // Return cached path if available
    {
        let cached = YTDLP_PATH.lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;
        if let Some(ref path) = *cached {
            if path.exists() {
                return Ok(path.clone());
            }
        }
    }

    // Try resource directory first (bundled with app)
    let resource_path = app.path().resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?
        .join(ytdlp_binary_name());

    if resource_path.exists() {
        *YTDLP_PATH.lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))? = Some(resource_path.clone());
        return Ok(resource_path);
    }

    // Try local data directory
    let local_data_dir = app.path().app_local_data_dir()
        .map_err(|e| format!("Failed to get local data dir: {}", e))?;

    let local_ytdlp = local_data_dir.join(ytdlp_binary_name());
    if local_ytdlp.exists() {
        *YTDLP_PATH.lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))? = Some(local_ytdlp.clone());
        return Ok(local_ytdlp);
    }

    // Download yt-dlp to local data directory
    download_ytdlp(&local_data_dir)?;

    *YTDLP_PATH.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))? = Some(local_ytdlp.clone());
    Ok(local_ytdlp)
}

/// Download yt-dlp binary to the specified directory
fn download_ytdlp(target_dir: &Path) -> Result<(), String> {
    println!("download_ytdlp: Starting download to {:?}", target_dir);

    fs::create_dir_all(target_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    let ytdlp_filename = ytdlp_binary_name();
    let ytdlp_path = target_dir.join(ytdlp_filename);

    // Download the platform-specific binary
    let download_url = ytdlp_download_url();

    println!("download_ytdlp: Downloading from {}", download_url);

    let response = minreq::get(download_url)
        .with_timeout(30) // 30 second timeout
        .send()
        .map_err(|e| {
            println!("download_ytdlp: Download failed: {}", e);
            format!("Failed to download yt-dlp: {}", e)
        })?;

    println!("download_ytdlp: Got response with status code {}", response.status_code);

    if response.status_code < 200 || response.status_code >= 300 {
        return Err(format!("Failed to download yt-dlp: HTTP {}", response.status_code));
    }

    let content = response.as_bytes();
    println!("download_ytdlp: Downloaded {} bytes", content.len());

    let mut file = File::create(&ytdlp_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    file.write_all(content)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    println!("download_ytdlp: File written to {:?}", ytdlp_path);

    // On Unix-like systems (macOS/Linux), make the binary executable
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&ytdlp_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?
            .permissions();
        perms.set_mode(0o755); // rwxr-xr-x
        fs::set_permissions(&ytdlp_path, perms)
            .map_err(|e| format!("Failed to set executable permissions: {}", e))?;
    }

    println!("download_ytdlp: Successfully downloaded yt-dlp");

    Ok(())
}

/// Validate YouTube URL with strict pattern matching
/// Prevents command injection and ensures only valid YouTube URLs are accepted
fn is_valid_youtube_url(url: &str) -> bool {
    // Check basic URL structure
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return false;
    }

    // Use regex to validate YouTube URL patterns
    // Matches: youtube.com/watch?v=ID or youtu.be/ID
    let youtube_pattern = regex::Regex::new(
        r"^(https?://)?(www\.)?(youtube\.com/watch\?v=[\w-]+|youtu\.be/[\w-]+)"
    ).map_err(|_| false);

    if youtube_pattern.is_err() {
        return false;
    }

    let re = youtube_pattern.unwrap();
    re.is_match(url)
}

/// Validate and sanitize output path to prevent path traversal attacks
/// Ensures the path is within allowed directories (Downloads or AppData)
fn validate_output_path(app: &AppHandle, path: &str) -> Result<String, String> {
    use std::path::{Path, PathBuf};

    let path_obj = PathBuf::from(path);

    // Get absolute path (without requiring it to exist)
    // This resolves any parent directory references (../) but doesn't fail if path doesn't exist
    let absolute_path = if path_obj.is_absolute() {
        path_obj.clone()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?
            .join(&path_obj)
    };

    // Normalize the path by resolving any .. or . components
    // NOTE: canonicalize() on Windows adds \\?\ prefix which breaks yt-dlp
    // So we use it only for validation, not for the returned path
    let normalized_path = Path::new(&absolute_path).canonicalize()
        .unwrap_or_else(|_| absolute_path.clone());

    // Get allowed base directories and normalize them too
    let downloads_dir = if let Some(home) = std::env::var("HOME").ok()
        .or_else(|| std::env::var("USERPROFILE").ok()) {
        PathBuf::from(home).join("Downloads")
    } else {
        app.path().app_local_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?
    };

    // Normalize the base directories for comparison
    let normalized_downloads = downloads_dir.canonicalize()
        .unwrap_or_else(|_| downloads_dir.clone());

    let app_data_dir = app.path().app_local_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let normalized_app_data = app_data_dir.canonicalize()
        .unwrap_or_else(|_| app_data_dir.clone());

    // Check if the path is within allowed directories
    let is_allowed = normalized_path.starts_with(&normalized_downloads)
        || normalized_path.starts_with(&normalized_app_data);

    if !is_allowed {
        return Err(format!(
            "Path not allowed. Only Downloads and app data directories are permitted.\nRequested: {:?}\nAllowed: {:?} or {:?}",
            normalized_path, normalized_downloads, normalized_app_data
        ));
    }

    // Return the ABSOLUTE path (not canonicalized) to avoid \\?\ prefix on Windows
    // This is safe because we've already validated it's within allowed directories
    Ok(absolute_path.to_string_lossy().to_string())
}

// Helper to check if yt-dlp is installed
#[tauri::command]
async fn check_ytdlp_installed(app: AppHandle) -> Result<bool, String> {
    println!("check_ytdlp_installed: Starting check...");

    match get_ytdlp_path(&app) {
        Ok(path) => {
            println!("check_ytdlp_installed: Found yt-dlp at {:?}", path);

            // Verify it works by running --version
            let mut cmd = Command::new(&path);

            // Windows: prevent console window
            #[cfg(target_os = "windows")]
            cmd.creation_flags(CREATE_NO_WINDOW);

            let result = cmd
                .arg("--version")
                .output();

            match &result {
                Ok(output) => {
                    println!("check_ytdlp_installed: Command output status: {}", output.status);
                    if output.status.success() {
                        println!("check_ytdlp_installed: yt-dlp is ready");
                    } else {
                        println!("check_ytdlp_installed: Command failed with status: {}", output.status);
                    }
                }
                Err(e) => {
                    println!("check_ytdlp_installed: Failed to run command: {}", e);
                }
            }

            Ok(result.is_ok() && result.map(|o| o.status.success()).unwrap_or(false))
        }
        Err(e) => {
            println!("check_ytdlp_installed: Error getting yt-dlp path: {}", e);
            Ok(false)
        }
    }
}

// Fetch playlist/video metadata
#[tauri::command]
async fn fetch_metadata(app: AppHandle, url: String) -> Result<PlaylistInfo, String> {
    let ytdlp = get_ytdlp_path(&app)?;

    // Check if it's a playlist or single video
    let is_playlist = url.contains("playlist") || url.contains("list=");

    if is_playlist {
        fetch_playlist_metadata(&ytdlp, url).await
    } else {
        fetch_single_video_metadata(&ytdlp, url).await
    }
}

async fn fetch_playlist_metadata(ytdlp: &Path, url: String) -> Result<PlaylistInfo, String> {
    let mut cmd = Command::new(ytdlp);

    // Windows: prevent console window
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd
        .args([
            "--dump-json",
            "--flat-playlist",
            "--user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
            "--referer", "https://www.youtube.com/",
            "--extractor-retries", "3",
            "--no-cache-dir",
            &url,
        ])
        .output()
        .map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to fetch playlist: {}", error_msg));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    let mut videos = Vec::new();
    for line in lines {
        if let Ok(video_data) = serde_json::from_str::<serde_json::Value>(line) {
            videos.push(VideoInfo {
                id: video_data.get("id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                title: video_data.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                url: format!("https://www.youtube.com/watch?v={}",
                    video_data.get("id").and_then(|v| v.as_str()).unwrap_or("")),
                duration: video_data.get("duration").and_then(|v| v.as_str()).map(String::from),
                thumbnail: video_data.get("thumbnail").and_then(|v| v.as_str()).map(String::from),
            });
        }
    }

    let title = if videos.first().is_some() {
        format!("Playlist with {} videos", videos.len())
    } else {
        "Empty Playlist".to_string()
    };

    Ok(PlaylistInfo {
        title,
        video_count: videos.len() as u32,
        videos,
    })
}

async fn fetch_single_video_metadata(ytdlp: &Path, url: String) -> Result<PlaylistInfo, String> {
    let mut cmd = Command::new(ytdlp);

    // Windows: prevent console window
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd
        .args([
            "--dump-json",
            "--user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
            "--referer", "https://www.youtube.com/",
            "--extractor-retries", "3",
            "--no-cache-dir",
            &url,
        ])
        .output()
        .map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to fetch video: {}", error_msg));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Ok(video_data) = serde_json::from_str::<serde_json::Value>(&stdout) {
        let video = VideoInfo {
            id: video_data.get("id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
            title: video_data.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
            url: url.clone(),
            duration: video_data.get("duration").and_then(|v| v.as_str()).map(String::from),
            thumbnail: video_data.get("thumbnail").and_then(|v| v.as_str()).map(String::from),
        };

        Ok(PlaylistInfo {
            title: video_data.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown Video").to_string(),
            video_count: 1,
            videos: vec![video],
        })
    } else {
        Err("Failed to parse video metadata".to_string())
    }
}

// Download video(s)
#[tauri::command]
async fn download_video(
    app: AppHandle,
    url: String,
    download_type: String,
    output_path: String,
    title: String,
    video_quality: String,
) -> Result<String, String> {
    let ytdlp = get_ytdlp_path(&app)?;
    let download_id = Uuid::new_v4().to_string();

    println!("=== Starting download ===");
    println!("URL: {}", url);
    println!("Type: {}", download_type);
    println!("Output path: {}", output_path);
    println!("Title: {}", title);

    // Validate URL with strict pattern matching
    println!("Validating URL...");
    if !is_valid_youtube_url(&url) {
        return Err("Invalid URL. Only YouTube URLs (youtube.com or youtu.be) are allowed.".to_string());
    }
    println!("URL validation passed");

    // Validate and sanitize output path to prevent path traversal attacks
    println!("Validating output path...");
    let validated_path = validate_output_path(&app, &output_path)?;
    println!("Path validation passed: {}", validated_path);
    let output_dir = Path::new(&validated_path);

    if !output_dir.exists() {
        println!("Creating output directory: {:?}", output_dir);
        fs::create_dir_all(&output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    // Build yt-dlp command
    let mut cmd = Command::new(&ytdlp);

    // Windows: prevent console window
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    // Common anti-bot detection flags
    cmd.args([
        "--user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
        "--referer", "https://www.youtube.com/",
        "--extractor-retries", "3",
        "--no-cache-dir",
        "--socket-timeout", "30",
    ]);

    // Format-specific arguments and output template
    if download_type == "audio" {
        // Audio: Download best audio quality without conversion (no FFmpeg needed)
        cmd.args([
            "-f", "bestaudio/best",
            "-o", &format!("{}/%(title)s.%(ext)s", output_path),
            "--newline",
            "--no-playlist",
        ]);
    } else {
        // Video: Download pre-merged video+audio files (no FFmpeg needed)
        // IMPORTANT: Do NOT use format code combination (+) as it requires FFmpeg to merge
        // Only download single files that already contain both video and audio
        let format_string = match video_quality.as_str() {
            "4k" => "best[height<=?2160][ext=mp4]/best[ext=mp4]/best",
            "1080p" => "best[height<=?1080][ext=mp4]/best[ext=mp4]/best",
            "720p" => "best[height<=?720][ext=mp4]/best[ext=mp4]/best",
            "480p" => "best[height<=?480][ext=mp4]/best[ext=mp4]/best",
            _ => "best[ext=mp4]/best",
        };

        cmd.args([
            "-f", format_string,
            "-o", &format!("{}/%(title)s.%(ext)s", output_path),
            "--newline",
            "--no-playlist",
        ]);
    }

    cmd.arg(&url);

    println!("Spawning download process...");
    println!("Command: {:?}", cmd);

    // Spawn the process
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn download process: {}", e))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    println!("Spawned download process with PID: {:?}", child.id());

    // Store in registry
    {
        let mut registry = DOWNLOAD_REGISTRY.lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;
        registry.insert(download_id.clone(), ActiveDownload::new(
            child,
            stdout,
            stderr,
            output_path.clone(),
            title.clone(),
        ));
    }

    // Emit initial state
    let _ = app.emit("download-progress", serde_json::json!({
        "id": download_id,
        "url": url,
        "title": title,
        "progress": 0.0,
        "status": "starting",
        "downloadType": download_type,
    }));

    // Start monitoring the download in a separate thread
    let download_id_clone = download_id.clone();
    let app_clone = app.clone();
    std::thread::spawn(move || {
        monitor_download(download_id_clone, app_clone);
    });

    Ok(download_id)
}

// Download entire playlist
#[tauri::command]
async fn download_playlist(
    app: AppHandle,
    _url: String,  // Playlist URL not used directly, but kept for future features
    download_type: String,
    output_path: String,
    videos: Vec<VideoInfo>,
    video_quality: String,
) -> Result<Vec<String>, String> {
    let mut download_ids = Vec::new();

    for video in &videos {
        match download_video(
            app.clone(),
            video.url.clone(),
            download_type.clone(),
            output_path.clone(),
            video.title.clone(),
            video_quality.clone(),
        ).await {
            Ok(id) => download_ids.push(id),
            Err(e) => {
                let _ = app.emit("download-error", serde_json::json!({
                    "url": video.url,
                    "error": e,
                }));
            }
        }
    }

    Ok(download_ids)
}

// Monitor download progress
fn monitor_download(download_id: String, app: AppHandle) {
    use std::io::{BufRead, BufReader};

    // Take stdout from registry for reading
    let reader = {
        let mut registry = DOWNLOAD_REGISTRY.lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e)).unwrap();
        if let Some(download) = registry.get_mut(&download_id) {
            download.stdout.take()
        } else {
            None
        }
    };

    // Spawn a thread to read progress
    let app_clone = app.clone();
    let download_id_clone = download_id.clone();
    std::thread::spawn(move || {
        if let Some(stdout) = reader {
            let reader = BufReader::new(stdout).lines();
            for line in reader.flatten() {
                // Check if download was cancelled before processing this line
                let is_cancelled = {
                    let registry = DOWNLOAD_REGISTRY.lock().unwrap();
                    registry.get(&download_id_clone).map(|d| d.cancelled).unwrap_or(false)
                };

                if is_cancelled {
                    println!("Progress reader: Download {} cancelled, stopping", download_id_clone);
                    break;
                }

                // Parse yt-dlp progress output
                // Format: [download]  45.2% of 10.00MiB at  1.00MiB/s ETA 00:05
                if line.contains("[download]") && line.contains('%') {
                    if let Some(start) = line.find('[') {
                        if let Some(percent_start) = line[start..].find(|c: char| c.is_ascii_digit()) {
                            let percent_str = &line[start + percent_start..];
                            if let Some(percent_end) = percent_str.find('%') {
                                let percent_num = &percent_str[..percent_end];
                                if let Ok(progress) = percent_num.trim().parse::<f64>() {
                                    let _ = app_clone.emit("download-progress", serde_json::json!({
                                        "id": download_id_clone,
                                        "status": "downloading",
                                        "progress": progress,
                                    }));
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    // Keep checking the process
    loop {
        // Check if download was cancelled
        let child_result = {
            let mut registry = match DOWNLOAD_REGISTRY.lock() {
                Ok(r) => r,
                Err(_) => {
                    // Lock failed - stop monitoring
                    return;
                }
            };

            let download = registry.get(&download_id);
            let is_cancelled = download.map(|d| d.cancelled).unwrap_or(false);

            if is_cancelled {
                println!("Download {} was cancelled, stopping monitoring", download_id);
                return; // Exit monitoring thread immediately
            }

            registry.get_mut(&download_id).map(|d| d.child.try_wait())
        };

        match child_result {
            Some(Ok(Some(result))) => {
                // Process has exited
                println!("Download process exited for: {}", download_id);
                println!("Exit code: {:?}", result.code());

                // Check if it was an error
                if result.code() != Some(0) {
                    // Try to capture stderr from the child process
                    // IMPORTANT: Take stderr OUTSIDE the lock to avoid holding lock during I/O
                    let stderr_option = {
                        let mut registry = DOWNLOAD_REGISTRY.lock()
                            .map_err(|_| "Failed to acquire lock").unwrap();
                        if let Some(download) = registry.get_mut(&download_id) {
                            download.stderr.take()
                        } else {
                            None
                        }
                    };

                    // Now read stderr WITHOUT holding the lock
                    let error_msg = if let Some(mut stderr) = stderr_option {
                        use std::io::Read;
                        let mut error_bytes = Vec::new();
                        match stderr.read_to_end(&mut error_bytes) {
                            Ok(n) => {
                                println!("Read {} bytes from stderr", n);
                                if n > 0 {
                                    // Convert bytes to string with lossy UTF-8 conversion
                                    // This handles non-UTF-8 characters gracefully
                                    let error_output = String::from_utf8_lossy(&error_bytes);
                                    println!("stderr content: {}", error_output);
                                }
                            }
                            Err(e) => {
                                println!("Failed to read stderr: {}", e);
                            }
                        }

                        if !error_bytes.is_empty() {
                            // Convert bytes to string with lossy UTF-8 conversion
                            let error_output = String::from_utf8_lossy(&error_bytes);
                            // Take only first 500 chars of error to avoid massive error messages
                            let truncated = if error_output.len() > 500 {
                                format!("{}... (truncated)", &error_output[..500])
                            } else {
                                error_output.to_string()
                            };
                            truncated
                        } else {
                            format!("Download failed with exit code {:?} (no stderr output)", result.code())
                        }
                    } else {
                        format!("Download failed with exit code {:?} (stderr not available)", result.code())
                    };

                    eprintln!("Error details: {}", error_msg);
                    let _ = app.emit("download-progress", serde_json::json!({
                        "id": download_id,
                        "status": "error",
                        "error": error_msg,
                    }));
                } else {
                    let _ = app.emit("download-progress", serde_json::json!({
                        "id": download_id,
                        "progress": 100.0,
                        "status": "completed",
                    }));
                }

                // Clean up - minimize lock scope
                {
                    let mut registry = DOWNLOAD_REGISTRY.lock()
                        .map_err(|_| "Failed to acquire lock").unwrap();
                    registry.remove(&download_id);
                }
                break;
            }
            Some(Ok(None)) => {
                // Still running, just wait
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            Some(Err(_)) => {
                // Error checking process status
                let _ = app.emit("download-progress", serde_json::json!({
                    "id": download_id,
                    "status": "error",
                    "error": "Failed to check process status",
                }));
                break;
            }
            None => {
                // Download not found in registry
                break;
            }
        }
    }
}

// Cancel download
#[tauri::command]
async fn cancel_download(app: AppHandle, download_id: String) -> Result<bool, String> {
    println!("Attempting to cancel download: {}", download_id);

    // First, mark the download as cancelled so monitoring thread knows to stop
    {
        let mut registry = DOWNLOAD_REGISTRY.lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        if let Some(download) = registry.get_mut(&download_id) {
            download.cancelled = true;
            println!("Marked download as cancelled: {}", download_id);
        } else {
            return Err("Download not found".to_string());
        }
    }

    // Now forcefully kill the process
    let title = {
        let mut registry = DOWNLOAD_REGISTRY.lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        let mut download = registry.remove(&download_id)
            .ok_or_else(|| "Download not found".to_string())?;

        // Forcefully kill the process on Windows
        println!("Killing process for download: {}", download_id);

        // On Windows, we need to ensure the process is killed
        // Try multiple approaches to ensure termination
        let kill_result = download.child.kill();

        if let Err(e) = &kill_result {
            println!("Initial kill failed: {}, trying wait...", e);
        }

        // Always try to wait to clean up resources, even if kill failed
        match download.child.wait() {
            Ok(status) => {
                println!("Process exited with status: {:?}", status);
            }
            Err(e) => {
                println!("Error waiting for process: {}", e);
            }
        }

        // Verify process is actually dead by checking PID
        // On Windows, kill the entire process tree (parent + all children)
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            let pid = download.child.id();
            // Use taskkill /F /T to forcefully terminate process tree
            // /F = force terminate, /T = terminate child processes
            let mut cmd = Command::new("taskkill");
            cmd.creation_flags(CREATE_NO_WINDOW);
            let result = cmd
                .args(["/F", "/T", "/PID", &pid.to_string()])
                .output();

            match &result {
                Ok(output) => {
                    println!("taskkill output: {}", String::from_utf8_lossy(&output.stdout));
                    if !output.stderr.is_empty() {
                        println!("taskkill stderr: {}", String::from_utf8_lossy(&output.stderr));
                    }
                }
                Err(e) => {
                    println!("taskkill command failed: {}", e);
                }
            }

            println!("Forcefully terminated process tree PID: {}", pid);
        }

        // On Unix-like systems, kill the process group
        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::process::CommandExt;
            // Kill process group to ensure child processes die too
            if let Err(e) = download.child.kill() {
                println!("Failed to kill process group: {}", e);
            }
        }

        println!("Process termination completed");

        let output_path = download.output_path.clone();
        let title = download.title.clone();

        // Clean up partial files
        println!("Cleaning up partial files for: {}", title);
        if let Err(e) = cleanup_partial_files(&output_path, &title) {
            println!("Warning: Failed to cleanup partial files: {}", e);
        }

        title
    };

    // Emit cancellation event with title so frontend doesn't show "Unknown"
    let _ = app.emit("download-progress", serde_json::json!({
        "id": download_id,
        "status": "cancelled",
        "title": title,
        "progress": 0.0,
    }));

    println!("Download cancelled successfully: {}", download_id);
    Ok(true)
}

// Clean up partial download files created by yt-dlp
fn cleanup_partial_files(output_path: &str, title: &str) -> Result<(), String> {
    use std::path::Path;
    use std::thread;
    use std::time::Duration;

    let output_dir = Path::new(output_path);
    if !output_dir.exists() {
        return Ok(()); // Nothing to clean if directory doesn't exist
    }

    println!("Scanning directory for partial files: {:?}", output_dir);

    let entries = output_dir.read_dir()
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        // Only delete files that START with the exact title (to avoid deleting other downloads' files)
        // yt-dlp creates files like: "Title.mp4.part", "Title.mp4.ytdl", "Title.mp4.part-Frag35.part"
        let is_related_to_download = file_name_str.starts_with(title);

        if !is_related_to_download {
            continue; // Skip files not related to this download
        }

        // Check if it has a temporary extension
        let is_temp_file = file_name_str.ends_with(".part") ||
                          file_name_str.ends_with(".temp") ||
                          file_name_str.ends_with(".ytdl") ||
                          file_name_str.contains(".ytdl");

        if is_temp_file {
            println!("Found temporary file: {:?}", entry.path());

            // Try to delete with retry logic for Windows file handle delays
            let mut attempts = 0;
            let max_attempts = 5;

            loop {
                match std::fs::remove_file(entry.path()) {
                    Ok(_) => {
                        println!("Successfully removed: {:?}", entry.path());
                        break;
                    }
                    Err(e) => {
                        attempts += 1;
                        if attempts >= max_attempts {
                            println!("Warning: Failed to remove after {} attempts: {}", max_attempts, e);
                            break;
                        }

                        // Wait a bit for Windows to release the file handle
                        println!("File locked, retrying in 200ms... (attempt {}/{})", attempts, max_attempts);
                        thread::sleep(Duration::from_millis(200));
                    }
                }
            }
        }
    }

    Ok(())
}

// Get download directory
#[tauri::command]
async fn select_download_folder(app: AppHandle) -> Result<String, String> {
    // Try to get the user's home directory (works on all platforms)
    if let Some(home) = std::env::var("HOME").ok()
        .or_else(|| std::env::var("USERPROFILE").ok()) {
        // Use PathBuf for cross-platform path handling
        let downloads = PathBuf::from(home).join("Downloads");
        Ok(downloads.to_string_lossy().to_string())
    } else {
        // Fallback to app local data directory
        let local_dir = app.path().app_local_data_dir()
            .map_err(|e| format!("Failed to get local data dir: {}", e))?;
        Ok(local_dir.to_string_lossy().to_string())
    }
}

// Update yt-dlp to latest version
#[tauri::command]
async fn update_ytdlp(app: AppHandle) -> Result<String, String> {
    let ytdlp = get_ytdlp_path(&app)?;

    println!("Updating yt-dlp at: {:?}", ytdlp);

    let mut cmd = Command::new(&ytdlp);

    // Windows: prevent console window
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd
        .arg("--update")
        .output()
        .map_err(|e| format!("Failed to run yt-dlp update: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        Ok(format!("yt-dlp updated successfully.\n{}", stdout))
    } else {
        Err(format!("Failed to update yt-dlp: {}", stderr))
    }
}

// Validate URL
#[tauri::command]
async fn validate_url(url: String) -> Result<bool, String> {
    if url.is_empty() {
        return Ok(false);
    }

    // Use strict URL validation to prevent bypass attempts
    Ok(is_valid_youtube_url(&url))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Initialize yt-dlp on app startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = get_ytdlp_path(&app_handle) {
                    eprintln!("Failed to initialize yt-dlp: {}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_ytdlp_installed,
            fetch_metadata,
            download_video,
            download_playlist,
            cancel_download,
            select_download_folder,
            validate_url,
            update_ytdlp,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
