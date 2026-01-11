# Verdl

<div align="center">

![Verdl Logo](/Verdl.png)

**A modern, elegant YouTube downloader built with Tauri**

![Tauri](https://img.shields.io/badge/Tauri-2.0-FFC131?style=for-the-badge&logo=tauri)
![Rust](https://img.shields.io/badge/Rust-1.81+-000000?style=for-the-badge&logo=rust)
![Cross-platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-blue?style=for-the-badge)
![License](https://img.shields.io/badge/License-MIT-239120?style=for-the-badge)

[Features](#features) • [Installation](#installation) • [Usage](#usage) • [FAQ](#faq) • [Troubleshooting](#troubleshooting)

</div>

---

## Overview

**Verdl** is a professional desktop application for downloading YouTube videos and playlists. Built with [Tauri 2.0](https://tauri.app/) and powered by [yt-dlp](https://github.com/yt-dlp/yt-dlp), it provides a clean, intuitive interface with real-time progress tracking and robust download management.

### Design Philosophy

Verdl embraces **refined minimalism** with professional polish:
- Clean typography using Inter and JetBrains Mono
- Thoughtful spacing and visual hierarchy
- Smooth micro-interactions and animations
- High contrast for excellent readability
- No unnecessary clutter - every element has a purpose

---

## Features

### Core Functionality
- **Single Video & Playlist Downloads** - Download individual videos or entire YouTube playlists
- **Video Quality Options** - Choose from 4K, 1080p, 720p, or 480p
- **Audio Extraction** - Convert videos to high-quality MP3 format
- **Concurrent Downloads** - Control how many videos download simultaneously (1-5)
- **Real-Time Progress** - Live download progress with detailed status updates
- **Smart Cancellation** - Cancel individual downloads or all at once with proper cleanup

### User Experience
- **Metadata Preview** - View playlist/video information before downloading
- **Select All/Deselect All** - Quickly choose which videos to download
- **Organized Downloads Section** - Track all active downloads in one place
- **Toast Notifications** - Clear feedback for all actions
- **Responsive Design** - Clean, professional interface that works at any window size

### Advanced Features
- **yt-dlp Updates** - Built-in "Update yt-dlp" button in header to keep yt-dlp current
- **Anti-Bot Detection** - Uses browser headers and referer to avoid YouTube's automated download restrictions
- **Smart Path Handling** - Automatic path validation with security checks
- **Partial File Cleanup** - Removes incomplete files when downloads are cancelled

---

## Tech Stack

### Frontend
- **HTML5** - Semantic markup
- **CSS3** - Custom styling with CSS variables for theming
- **Vanilla JavaScript (ES6+)** - No framework dependencies
- **Tauri API 2.0** - Native system integration

### Backend
- **Rust** - Performance and safety
- **Tauri 2.0** - Cross-platform desktop framework
- **yt-dlp** - YouTube media downloader engine

### Dependencies

**Rust (Cargo.toml)**:
- `tauri` 2.0 - Desktop framework
- `serde` 1 - Serialization
- `uuid` 1 - Unique identifiers
- `once_cell` 1 - Static state management
- `regex` 1 - URL validation

**Node.js (package.json)**:
- `@tauri-apps/api` 2.0 - Tauri frontend API
- `@tauri-apps/cli` 2.0 - Build tooling

---

## Prerequisites

### Required Software

#### 1. Rust (1.81 or later)

**Windows:**
```bash
winget install Rustlang.Rustup
# Or download from: https://rustup.rs/
```

**macOS:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### 2. Node.js (v20 or later)

**Windows:**
```bash
winget install OpenJS.NodeJS.LTS
# Or download from: https://nodejs.org/
```

**macOS:**
```bash
brew install node
# Or download from: https://nodejs.org/
```

**Linux:**
```bash
# Ubuntu/Debian
sudo apt install nodejs npm

# Fedora
sudo dnf install nodejs npm

# Or download from: https://nodejs.org/
```

#### 3. yt-dlp (Auto-Downloaded)

**Good news:** Verdl automatically downloads yt-dlp on first run! No manual installation needed.

**Optional:** If you want to install yt-dlp manually:

**All platforms:**
```bash
pip install yt-dlp
```

**Verify installation:**
```bash
yt-dlp --version
```

### Verify Installation

```bash
rustc --version  # Should output rustc 1.81.x or later
node --version   # Should output v20.x.x or later
yt-dlp --version # Should output yt-dlp version
```

---

## Installation

### Quick Start

```bash
# Clone the repository
git clone <repository-url>
cd yt-dlp-downloader

# Install Node.js dependencies
npm install

# Run in development mode
npm run dev
```

### Development Mode

```bash
npm run dev
```

This launches the app with hot reload for frontend changes and automatic rebuilds for Rust changes.

### Production Build

```bash
npm run build
```

The built executable will be in:
```
src-tauri/target/release/bundle/nsis/
```

---

## Usage

### Basic Workflow

#### 1. Check yt-dlp Status
When you launch Verdl, check the status indicator in the top-right corner:
- **Green dot** = yt-dlp is installed and ready
- **Red dot** = yt-dlp not found (install it first)

#### 2. Enter YouTube URL
Paste a YouTube video or playlist URL in the input field and press **Enter** or click **Fetch**.

**Supported URLs**:
- Single videos: `https://www.youtube.com/watch?v=...`
- Playlists: `https://www.youtube.com/playlist?list=...`
- Shortened: `https://youtu.be/...`

#### 3. Preview Content
After fetching, you'll see:
- Playlist/video title
- Number of videos
- List of all videos with titles and durations

#### 4. Choose Format
- **Video** - Downloads as MP4 (best available quality)
- **Audio MP3** - Extracts high-quality audio only

For video, select quality:
- **4K** (2160p) - Ultra HD
- **1080p** (Full HD) - Default
- **720p** (HD) - Standard HD
- **480p** (SD) - Standard definition

#### 5. Select Videos (Optional)
- **Select All** - All videos are selected by default
- Click **Deselect All** to uncheck all videos
- Click individual videos to toggle selection

#### 6. Set Concurrency
Adjust the **Concurrent** slider (1-5):
- **1** (default) - Safest, least likely to trigger bot detection
- **5** - Faster downloads, but may trigger restrictions

#### 7. Download
Click **Download All** to start. Monitor progress in the sidebar under **Active Downloads**.

#### 8. Cancel Downloads (If Needed)
- Click **X** on individual download cards to cancel one
- Downloads can be cancelled anytime, partial files are cleaned up automatically

---

## FAQ

### Q: Does Verdl work with YouTube Music?
**A:** Yes! Verdl can download YouTube Music videos and extract audio. Just paste the YouTube Music URL.

### Q: Can I download age-restricted videos?
**A:** This depends on yt-dlp's capabilities. You may need to update yt-dlp using the "Update yt-dlp" button if you encounter issues.

### Q: Why do downloads fail with "Sign in to confirm you're not a bot"?
**A:** YouTube has anti-bot detection. Verdl includes anti-detection measures, but YouTube updates their systems frequently. When this happens:
1. Click the "Update yt-dlp" button that appears
2. Wait 10-15 minutes for rate limits to expire
3. Try again with concurrency set to 1

### Q: Does YouTube block based on IP address?
**A:** YouTube can rate-limit based on IP patterns, but the "bot detection" error is typically NOT IP-based. It's caused by:
- Missing browser headers (Verdl handles this)
- Outdated yt-dlp version (use the update button)
- Too many requests too quickly (lower concurrency)

### Q: Can I download private or unlisted videos?
**A:** No. Verdl only works with publicly accessible YouTube content.

### Q: Where are downloads saved?
**A:** By default, downloads go to your Windows Downloads folder:
```
C:\Users\YourUsername\Downloads\
```

### Q: Can I change the download location?
**A:** Currently, Verdl uses the system Downloads folder. Future versions may allow customization.

### Q: Does Verdl support subtitles?
**A:** Not currently. This feature may be added in future versions.

### Q: Can I download 8K videos?
**A:** The current maximum quality is 4K (2160p). 8K support may be added in the future.

### Q: Why does Verdl need yt-dlp installed separately?
**A:** yt-dlp is actively updated (sometimes daily) to bypass YouTube's restrictions. By using your system's yt-dlp, you can easily update it independently of Verdl.

### Q: Is Verdl free?
**A:** Yes! Verdl is open-source and free to use. yt-dlp is also free and open-source.

### Q: Does Verdl collect any data?
**A:** No. Verdl is a local desktop application that doesn't send data anywhere except to YouTube when downloading videos.

### Q: Can I use Verdl on macOS or Linux?
**A:** Yes! Verdl is cross-platform and works on:
- **Windows** (Windows 10+)
- **macOS** (macOS 10.15+)
- **Linux** (most distributions)

The app automatically downloads the correct yt-dlp binary for your platform.

---

## Troubleshooting

### Status Stuck on "Checking..."

**Symptom**: Status indicator shows "Checking..." and never changes

**Possible causes**:
1. yt-dlp is downloading for the first time (may take 30 seconds)
2. Network connectivity issues
3. yt-dlp download failed silently

**Solutions**:
1. **Wait 30 seconds** - Auto-download may be in progress
2. **Install yt-dlp manually**:
   ```bash
   pip install yt-dlp
   ```
3. **Check browser console** (F12) for error messages
4. **Restart the app** after installing yt-dlp

### "yt-dlp: NOT FOUND"

**Symptom**: Status indicator shows red dot

**Solution**:
```bash
# Install yt-dlp using pip
pip install yt-dlp

# Or download Windows binary from:
# https://github.com/yt-dlp/yt-dlp/releases/latest
# Place yt-dlp.exe in C:\Windows\System32\ or another PATH folder

# Verify installation
yt-dlp --version
```

### "Sign in to confirm you're not a bot" Error

**Symptom**: Downloads fail immediately with bot detection message

**Solutions** (try in order):
1. **Click the "Update yt-dlp" button** in the header (appears automatically)
2. **Wait 10-15 minutes** - Rate limits may need to expire
3. **Lower concurrency** to 1
4. **Try a different URL** - Some videos have stricter protection
5. **Update yt-dlp manually**:
   ```bash
   pip install --upgrade yt-dlp
   ```

See [BOT_DETECTION_SOLUTIONS.md](/BOT_DETECTION_SOLUTIONS.md) for detailed information.

### Download Stalls or Progress Stops

**Symptom**: Progress bar freezes, status doesn't change

**Possible causes**:
- Slow internet connection
- YouTube throttling
- Large file size

**Solution**:
- Wait a few minutes (some large files take time)
- Check your internet connection
- Try downloading a smaller/lower-quality video first

### "Failed to fetch metadata" Error

**Symptom**: Can't retrieve video/playlist information

**Solutions**:
1. **Check URL** - Make sure it's a valid YouTube URL
2. **Check internet connection**
3. **Update yt-dlp** - Use the update button or run: `pip install --upgrade yt-dlp`
4. **Try a different URL** - Some videos may be region-locked or private

### Build Errors

#### "error: linker `link.exe` not found"

**Solution**: Install MSVC (Microsoft Visual C++ build tools):
```bash
winget install Microsoft.VisualStudio.2022.BuildTools
```

Or install Visual Studio Community with C++ workload.

#### "cargo: command not found"

**Solution**: Rust is not installed or not in PATH. Reinstall Rust from https://rustup.rs/

#### "npm: command not found"

**Solution**: Node.js is not installed or not in PATH. Reinstall from https://nodejs.org/

### Application Won't Launch

**Symptom**: Built executable crashes immediately

**Solutions**:
1. **Check antivirus** - Some antivirus software flags Tauri apps
2. **Run as administrator** - Right-click → Run as administrator
3. **Check Event Viewer** - Look for crash logs in Windows Event Viewer
4. **Rebuild in debug mode**:
   ```bash
   npm run dev
   ```
   Check console for error messages

### High CPU Usage During Download

**Symptom**: Application uses lots of CPU while downloading

**This is normal** for yt-dlp when:
- Downloading high-quality video (4K)
- Converting formats
- Merging video and audio streams

**To reduce CPU usage**:
- Lower video quality (720p or 480p)
- Reduce concurrent downloads to 1
- Close other applications

### Partial Files Left in Download Folder

**Symptom**: .part, .temp, or .ytdl files remain after cancelled downloads

**Solution**: Verdl should clean these up automatically. If you see leftover files:
1. They're safe to delete manually
2. Check you're using the latest version of Verdl
3. Report this as a bug if it persists

---

## Project Structure

```
yt-dlp-downloader/
├── src/                       # Frontend
│   ├── index.html            # Main UI structure
│   ├── styles.css            # Application styling
│   └── main.js               # Application logic
├── src-tauri/                # Rust backend
│   ├── src/
│   │   ├── lib.rs           # Tauri commands & yt-dlp integration
│   │   └── main.rs          # Entry point
│   ├── resources/           # Bundled resources (yt-dlp, icons)
│   ├── Cargo.toml           # Rust dependencies
│   └── tauri.conf.json      # Tauri configuration
├── icons/                    # Application icons
├── README.md                 # This file
├── BOT_DETECTION_SOLUTIONS.md # Detailed bot detection info
└── package.json              # Node.js dependencies
```

---

## Development

### Adding New Features

#### Backend (Rust)

1. Open `src-tauri/src/lib.rs`
2. Add a new function with `#[tauri::command]` attribute:
```rust
#[tauri::command]
async fn my_new_command(param: String) -> Result<String, String> {
    // Your logic here
    Ok("Success".to_string())
}
```

3. Register the command in `invoke_handler`:
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    my_new_command,
])
```

#### Frontend (JavaScript)

1. Call the command using `invoke()`:
```javascript
const result = await invoke('my_new_command', { param: 'value' });
```

2. Update UI based on result

### Code Style Guidelines

- **Rust**: Follow standard rustfmt formatting
- **JavaScript**: Use ES6+ features, template literals, async/await
- **CSS**: Use CSS variables for theming, BEM naming for classes
- **Comments**: Document complex logic, explain "why" not "what"

### Testing

```bash
# Run development build with hot reload
npm run dev

# Check Rust code for errors
cd src-tauri
cargo check

# Run Rust tests (if any are added)
cargo test
```

### Debugging

**View Rust backend logs:**
- Terminal where you ran `npm run dev` shows all `println!` output
- Look for messages prefixed with:
  - `check_ytdlp_installed:` - Status checks
  - `download_ytdlp:` - Download progress
  - `Fetching...` - Metadata fetching

**View frontend logs:**
- Open DevTools with F12
- Console tab shows JavaScript errors and logs
- Network tab shows Tauri command invocations

**Common issues to check:**
1. Status stuck on "Checking..." - See "Status Stuck on 'Checking...'" in Troubleshooting
2. Downloads fail immediately - Check for bot detection errors
3. Console windows appear (Windows only) - Fixed with CREATE_NO_WINDOW flag

---

## Building for Distribution

### Windows

```bash
npm run build
```

The installer will be created at:
```
src-tauri/target/release/bundle/nsis/Verdl_0.1.0_x64-setup.exe
```

### macOS

```bash
npm run build
```

The app bundle will be created at:
```
src-tauri/target/release/bundle/macos/Verdl.app
```

**Note:** macOS builds require code signing for distribution. Unsigned apps will work but may show warnings.

### Linux

```bash
npm run build
```

The AppImage will be created at:
```
src-tauri/target/release/bundle/appimage/verdl_0.1.0_amd64.AppImage
```

**Debian package:**
```
src-tauri/target/release/bundle/deb/verdl_0.1.0_amd64.deb
```

### Customizing the Build

Edit `src-tauri/tauri.conf.json` to customize:
- App name and version
- Window size and behavior
- Icon and branding
- Permissions and security settings

### Bundling yt-dlp

To bundle yt-dlp with the application (recommended for distribution):

**Windows:**
1. Download `yt-dlp.exe` from [releases](https://github.com/yt-dlp/yt-dlp/releases)
2. Place it in `src-tauri/resources/yt-dlp.exe`
3. Rebuild the app

**macOS:**
1. Download `yt-dlp` (Unix binary) from [releases](https://github.com/yt-dlp/yt-dlp/releases)
2. Place it in `src-tauri/resources/yt-dlp`
3. Rebuild the app

**Linux:**
1. Download `yt-dlp` (Linux binary) from [releases](https://github.com/yt-dlp/yt-dlp/releases)
2. Place it in `src-tauri/resources/yt-dlp`
3. Rebuild the app

The app will automatically use the bundled yt-dlp if found.

---

## Portability

### Is Verdl Portable?

**Yes!** Verdl is fully portable on all platforms.

**All Platforms:**
- **Single executable/bundle** - No installation required
- **Auto-downloads yt-dlp** on first run (requires internet)
- **Works from USB drives, network folders, etc.**

Since a YouTube downloader requires internet anyway, the auto-download of yt-dlp on first run is not a limitation.

**Optional: Bundle yt-dlp for offline-first setup**
```bash
# Place the platform-specific binary in src-tauri/resources/
# Windows: yt-dlp.exe
# macOS: yt-dlp
# Linux: yt-dlp
```

This is optional and only useful if you want to distribute a self-contained package.

---

## Security

### Path Traversal Protection

Verdl validates all output paths to prevent directory traversal attacks. Only allowed directories are permitted for downloads.

### URL Validation

Only YouTube URLs (`youtube.com` and `youtu.be`) are accepted to prevent unauthorized access to other websites.

### Content Security Policy

The app uses strict CSP headers to prevent XSS attacks and ensure secure communication.

---

## License

MIT License - see LICENSE file for details.

---

## Credits

### Core Technologies
- **[Tauri](https://tauri.app/)** - Cross-platform desktop framework
- **[yt-dlp](https://github.com/yt-dlp/yt-dlp)** - YouTube media downloader
- **[Rust](https://www.rust-lang.org/)** - Backend programming language
- **[Node.js](https://nodejs.org/)** - Frontend build tooling

### Fonts
- **[Inter](https://fonts.google.com/specimen/Inter)** - UI font by Google Fonts
- **[JetBrains Mono](https://fonts.jetbrains.com/jetbrains-mono)** - Monospace font by JetBrains

### Icons
- Custom Verdl logo and icons

---

## Contributing

Contributions are welcome! Please follow these guidelines:

1. **Fork** the repository
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes (`git commit -m 'Add amazing feature'`)
4. **Push** to the branch (`git push origin feature/amazing-feature`)
5. **Open a Pull Request**

### Code Review Standards
- Code must pass `cargo check` without warnings
- JavaScript should use modern ES6+ syntax
- CSS should follow existing naming conventions
- All features should include error handling

---

## Changelog

### Version 0.1.0 (Current)

#### Initial Release Features
- Single video and playlist downloads
- Video quality selection (4K to 480p)
- Audio extraction to MP3
- Concurrent downloads control (1-5)
- Real-time progress tracking
- Download cancellation with cleanup
- yt-dlp auto-update feature
- Anti-bot detection headers
- Metadata preview
- Select/deselect all videos

#### Recent Improvements
- ✅ **Cross-platform support** - Now works on Windows, macOS, and Linux
- ✅ **Console window fix** - No more command windows popping up on Windows
- ✅ **Improved error handling** - Better debugging with detailed logs
- ✅ **Download timeout** - 30-second timeout prevents hanging downloads
- ✅ **Platform-specific binaries** - Automatically downloads correct yt-dlp for your OS
- ✅ **Consistent UI** - Update button uses green theme color
- ✅ **Bundle identifier** - Fixed macOS bundle naming conflict

---

## Support

For issues, questions, or feature requests:
- Check the [Troubleshooting](#troubleshooting) section
- Read [BOT_DETECTION_SOLUTIONS.md](/BOT_DETECTION_SOLUTIONS.md) for bot detection help
- Open an issue on GitHub

---

## Roadmap

### Planned Features
- [ ] Custom download folder selection
- [ ] Download history tracking
- [ ] Subtitle download support
- [ ] Playlist channel support
- [ ] Dark/light theme toggle
- [ ] Batch URL import
- [ ] Download queue management
- [ ] Automatic retry on failure
- [ ] Format conversion options

---

<div align="center">

**Built with ❤️ using [Tauri](https://tauri.app/)**

*For digital preservation and easy access to online content*

</div>
