# YouTube Bot Detection Solutions

## Question: Does YouTube block based on IP?

**Short answer**: Yes, YouTube can block based on IP patterns, but it's typically **rate-limiting** rather than permanent blocking.

**Detailed answer**:
- YouTube uses multiple layers of bot detection (IP patterns, user-agent, request frequency, cookies)
- IP-based blocking is usually **temporary** (minutes to hours) for rate-limiting
- The "Sign in to confirm you're not a bot" error is **NOT typically IP-based** - it's a request pattern detection (missing cookies, wrong headers, automated tool signatures)
- IP-based blocks would result in connection timeouts or 429 errors, not the bot detection page

## Solutions Implemented

### 1. Anti-Bot Detection Flags (lib.rs:255-267, 304-315, 383-390)
Added to all yt-dlp commands:
```rust
"--user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
"--referer", "https://www.youtube.com/",
"--extractor-retries", "3",
"--no-cache-dir",
"--socket-timeout", "30",
```

**What this does**:
- Makes requests appear to come from a real Chrome browser
- Sets proper referer header (YouTube requires this)
- Retries failed extractions (bot detection may be intermittent)
- Disables cache (prevents stale detection states)
- Sets timeout to prevent hanging

### 2. yt-dlp Update Button (main.js:170-203, lib.rs:874-894)
Added "Update yt-dlp" button in header that:
- Only appears when bot detection is detected
- Runs `yt-dlp --update` to get latest version with new bypasses
- Shows success/error toast notifications
- Automatically hides after successful update

**Detection triggers** (main.js:146-151, 267-273):
- Error contains "bot"
- Error contains "sign in to confirm"

### 3. Reduced Concurrency (index.html:97-116, main.js:23)
Changed from 1-20 range to 1-5 range, default 1
- Less aggressive = less likely to trigger detection
- User can increase up to 5 if needed

## Why Cookies Approach Failed

The `--cookies-from-browser chrome` approach fails on Windows because:
- Chrome locks its cookie database while open
- Windows doesn't allow reading locked files
- yt-dlp cannot copy the database while Chrome is running

**Alternative cookie solutions** (not implemented):
1. Manual cookie export: Export cookies from browser → save to TXT file → use `--cookies file.txt`
2. Close Chrome before fetching (not user-friendly)
3. Use a portable browser profile

## Most Effective Solution

**Update yt-dlp regularly** - The yt-dlp team actively develops bypasses for YouTube's anti-bot measures. New versions are released frequently (sometimes daily) to counter YouTube's changes.

## Testing Checklist

When testing if a solution works:
1. Update yt-dlp using the new "Update yt-dlp" button
2. Try fetching metadata again
3. If still failing, wait 10-15 minutes (rate limit may expire)
4. Try a different YouTube URL (some videos have stricter protection)
5. Test from command line: `yt-dlp [URL]` (isolates if it's app-specific)

## What NOT to Do

- ❌ Don't spam requests (will trigger IP rate-limiting)
- ❌ Don't use VPN/proxy to bypass (YouTube may flag this as suspicious)
- ❌ Don't try to circumvent with cookie theft (violates ToS)
- ✅ Do update yt-dlp regularly
- ✅ Do wait between retries
- ✅ Do use the lowest concurrency that works

## Future Improvements

If bot detection persists:
1. Implement retry with exponential backoff
2. Add delay between playlist video fetches
3. Consider using YouTube Data API (requires API key, but more reliable)
4. Add user-agent rotation (multiple browser signatures)
