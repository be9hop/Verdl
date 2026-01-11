/**
 * ═════════════════════════════════════════════════════════════════
 * YT-DLP DOWNLOADER - PROFESSIONAL UI
 * Application Logic
 * ═════════════════════════════════════════════════════════════════
 */

// Use global Tauri API (injected at runtime in Tauri 2)
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// ═════════════════════════════════════════════════════════════════
// APPLICATION STATE
// ═════════════════════════════════════════════════════════════════

const state = {
  ytdlpInstalled: false,
  currentUrl: '',
  currentMetadata: null,
  selectedFormat: 'video',
  videoQuality: '1080p',
  outputPath: '',
  concurrentDownloads: 1,
  activeDownloads: new Map(),
  downloadCounter: 0,
  selectedVideos: new Set(), // Track which videos are selected for download
  cancelledDownloads: new Set(), // Track downloads that were cancelled by user
};

// ═════════════════════════════════════════════════════════════════
// DOM ELEMENTS
// ═════════════════════════════════════════════════════════════════

const elements = {
  // Status
  ytdlpStatus: document.getElementById('ytdlp-status'),
  statusDot: document.querySelector('.status-dot'),
  statusText: document.querySelector('.status-text'),
  updateYtdlpBtn: document.getElementById('update-ytdlp-btn'),

  // Input section
  urlInput: document.getElementById('url-input'),
  fetchBtn: document.getElementById('fetch-btn'),
  inputStatus: document.getElementById('input-status'),

  // Format & quality
  formatOptions: document.querySelectorAll('.format-option'),
  qualitySection: document.getElementById('quality-section'),
  qualitySelect: document.getElementById('quality-select'),

  // Concurrent slider
  concurrentSlider: document.getElementById('concurrent-slider'),
  concurrentValue: document.getElementById('concurrent-value'),

  // Output path
  outputPath: document.getElementById('output-path'),
  changePathBtn: document.getElementById('change-path-btn'),

  // Metadata panel
  metadataPanel: document.getElementById('metadata-panel'),
  emptyPanel: document.getElementById('empty-panel'),
  playlistTitle: document.getElementById('playlist-title'),
  videoCount: document.getElementById('video-count'),
  selectAllBtn: document.getElementById('select-all-btn'),
  videoList: document.getElementById('video-list'),

  // Action bar
  actionBar: document.getElementById('action-bar'),
  downloadAllBtn: document.getElementById('download-all-btn'),
  cancelBtn: document.getElementById('cancel-btn'),

  // Downloads
  downloadCount: document.getElementById('download-count'),
  downloadsList: document.getElementById('downloads-list'),

  // Toast
  toastContainer: document.getElementById('toast-container'),
};

// ═════════════════════════════════════════════════════════════════
// INITIALIZATION
// ═════════════════════════════════════════════════════════════════

async function initialize() {
  setupEventListeners();
  setupTauriEventListeners();
  await checkYtdlpStatus();
  await loadDownloadPath();
}

function setupEventListeners() {
  // URL input
  elements.urlInput.addEventListener('input', handleUrlInput);
  elements.urlInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter' && !elements.fetchBtn.disabled) {
      handleFetchMetadata();
    }
  });

  // Fetch metadata
  elements.fetchBtn.addEventListener('click', handleFetchMetadata);

  // Format selection
  elements.formatOptions.forEach(btn => {
    btn.addEventListener('click', handleFormatSelection);
  });

  // Quality selection
  elements.qualitySelect.addEventListener('change', handleQualityChange);

  // Concurrent downloads slider
  elements.concurrentSlider.addEventListener('input', handleConcurrentChange);

  // Change path button
  elements.changePathBtn.addEventListener('click', handleChangePath);

  // Update yt-dlp button
  elements.updateYtdlpBtn.addEventListener('click', handleUpdateYtdlp);

  // Select all/deselect all button
  elements.selectAllBtn.addEventListener('click', handleSelectAllToggle);

  // Download buttons
  elements.downloadAllBtn.addEventListener('click', handleDownloadAll);
  elements.cancelBtn.addEventListener('click', handleCancelDownloads);
}

function setupTauriEventListeners() {
  listen('download-progress', (event) => {
    const { id, progress, status, title, downloadType, converting } = event.payload;

    // If download was cancelled, remove it completely from UI
    if (status === 'cancelled') {
      removeDownload(id);
      return;
    }

    updateDownloadProgress(id, progress, status, title, downloadType, converting);
  });

  listen('download-error', (event) => {
    const { url, error } = event.payload;
    const errorMsg = error.toLowerCase();

    // Detect bot detection
    if (errorMsg.includes('bot') || errorMsg.includes('sign in to confirm')) {
      showToast('YouTube bot detected. Try updating yt-dlp.', 'warning');
    } else {
      showToast(`Download failed: ${error}`, 'error');
    }
  });
}

// ═════════════════════════════════════════════════════════════════
// YTDLP STATUS CHECK
// ═════════════════════════════════════════════════════════════════

async function checkYtdlpStatus() {
  try {
    console.log('Checking yt-dlp status...');
    const installed = await invoke('check_ytdlp_installed');
    console.log('yt-dlp installed result:', installed);
    state.ytdlpInstalled = installed;

    if (installed) {
      elements.statusDot.classList.add('active');
      elements.statusText.textContent = 'Ready';
    } else {
      elements.statusDot.classList.remove('active');
      elements.statusText.textContent = 'Not Found';
      showToast('yt-dlp not found. Installing...', 'warning');
    }
  } catch (error) {
    console.error('Failed to check yt-dlp status:', error);
    elements.statusText.textContent = 'Error';
    elements.statusDot.classList.remove('active');
    showToast(`Error checking yt-dlp: ${error}`, 'error');
  }
}

// Update yt-dlp to latest version
async function handleUpdateYtdlp() {
  try {
    elements.updateYtdlpBtn.disabled = true;
    elements.updateYtdlpBtn.innerHTML = `
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 2v6h-6"/>
        <path d="M3 12a9 9 0 0 1 15-6.7L21 8"/>
        <path d="M3 22v-6h6"/>
        <path d="M21 12a9 9 0 0 1-15 6.7L3 16"/>
      </svg>
      Updating...
    `;
    showToast('Checking for updates...', 'info');

    const result = await invoke('update_ytdlp');
    console.log('Update result:', result);

    // Check if yt-dlp was actually updated or already up to date
    const resultLower = result.toLowerCase();
    if (resultLower.includes('up to date') || resultLower.includes('already')) {
      showToast('yt-dlp is already up to date', 'success');
    } else {
      showToast('yt-dlp updated successfully!', 'success');
    }
  } catch (error) {
    showToast(`Failed to update yt-dlp: ${error}`, 'error');
    console.error('Update error:', error);
  } finally {
    elements.updateYtdlpBtn.disabled = false;
    elements.updateYtdlpBtn.innerHTML = `
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 2v6h-6"/>
        <path d="M3 12a9 9 0 0 1 15-6.7L21 8"/>
        <path d="M3 22v-6h6"/>
        <path d="M21 12a9 9 0 0 1-15 6.7L3 16"/>
      </svg>
      Update
    `;
  }
}

// ═════════════════════════════════════════════════════════════════
// URL INPUT HANDLING
// ═════════════════════════════════════════════════════════════════

function handleUrlInput(e) {
  const url = e.target.value.trim();
  state.currentUrl = url;

  const isValidUrl = url && (
    url.toLowerCase().includes('youtube.com') ||
    url.toLowerCase().includes('youtu.be')
  );

  elements.fetchBtn.disabled = !isValidUrl;
  elements.inputStatus.textContent = '';
  elements.inputStatus.className = 'input-status';
}

// ═════════════════════════════════════════════════════════════════
// METADATA FETCHING
// ═════════════════════════════════════════════════════════════════

async function handleFetchMetadata() {
  if (!state.ytdlpInstalled) {
    showToast('yt-dlp is not installed', 'error');
    return;
  }

  const url = state.currentUrl;
  if (!url) {
    showToast('Please enter a URL first', 'error');
    return;
  }

  elements.fetchBtn.disabled = true;
  elements.fetchBtn.innerHTML = `
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83"/>
    </svg>
    Fetching...
  `;
  elements.inputStatus.textContent = 'Retrieving metadata...';

  try {
    const metadata = await invoke('fetch_metadata', { url });
    state.currentMetadata = metadata;
    displayMetadata(metadata);
    showToast(`Found ${metadata.video_count} video(s)`, 'success');
  } catch (error) {
    console.error('Error fetching metadata:', error);
    elements.inputStatus.textContent = String(error);
    elements.inputStatus.classList.add('error');

    // Detect bot detection
    const errorMsg = String(error).toLowerCase();
    if (errorMsg.includes('bot') || errorMsg.includes('sign in to confirm')) {
      showToast('YouTube bot detected. Try updating yt-dlp.', 'warning');
    } else {
      showToast(`Failed to fetch metadata: ${error}`, 'error');
    }
  } finally {
    elements.fetchBtn.disabled = false;
    elements.fetchBtn.innerHTML = `
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="9 18 15 12 9 6"/>
      </svg>
      Fetch
    `;
  }
}

function displayMetadata(metadata) {
  elements.playlistTitle.textContent = metadata.title;
  elements.videoCount.textContent = `${metadata.video_count} videos`;

  // Reset and populate selected videos set (all selected by default)
  state.selectedVideos.clear();
  metadata.videos.forEach((video, index) => {
    state.selectedVideos.add(index);
  });

  // Set select all button state
  elements.selectAllBtn.textContent = 'Deselect All';

  // Clear and populate video list
  elements.videoList.innerHTML = '';
  metadata.videos.forEach((video, index) => {
    const videoItem = createVideoItem(video, index);
    elements.videoList.appendChild(videoItem);
  });

  // Show metadata panel and action bar
  elements.metadataPanel.style.display = 'flex';
  elements.emptyPanel.style.display = 'none';
  elements.actionBar.style.display = 'flex';

  // Update download button text
  updateDownloadButtonText();
}

function createVideoItem(video, index) {
  const div = document.createElement('div');
  div.className = 'video-item';
  div.dataset.index = index;

  // Create checkbox
  const checkbox = document.createElement('label');
  checkbox.className = 'video-checkbox';
  checkbox.innerHTML = `
    <input type="checkbox" ${state.selectedVideos.has(index) ? 'checked' : ''}>
    <span class="checkbox-custom"></span>
  `;

  // Handle checkbox change
  const input = checkbox.querySelector('input');
  input.addEventListener('change', () => {
    if (input.checked) {
      state.selectedVideos.add(index);
      div.classList.remove('excluded');
    } else {
      state.selectedVideos.delete(index);
      div.classList.add('excluded');
    }
    updateDownloadButtonText();
  });

  // Click on video item toggles checkbox (but not when clicking checkbox directly)
  div.addEventListener('click', (e) => {
    if (e.target !== input && e.target !== checkbox.querySelector('.checkbox-custom')) {
      input.checked = !input.checked;
      input.dispatchEvent(new Event('change'));
    }
  });

  div.innerHTML = '';
  div.appendChild(checkbox);

  // Add index
  const indexSpan = document.createElement('span');
  indexSpan.className = 'video-item-index';
  indexSpan.textContent = index + 1;
  div.appendChild(indexSpan);

  // Add title
  const titleSpan = document.createElement('span');
  titleSpan.className = 'video-item-title';
  titleSpan.textContent = video.title;
  titleSpan.title = video.title;
  div.appendChild(titleSpan);

  // Add duration if available
  if (video.duration) {
    const durationSpan = document.createElement('span');
    durationSpan.className = 'video-item-duration';
    durationSpan.textContent = video.duration;
    div.appendChild(durationSpan);
  }

  return div;
}

// ═════════════════════════════════════════════════════════════════
// FORMAT & QUALITY SELECTION
// ═════════════════════════════════════════════════════════════════

function handleFormatSelection(e) {
  const btn = e.currentTarget;
  const format = btn.dataset.format;

  elements.formatOptions.forEach(b => b.classList.remove('active'));
  btn.classList.add('active');

  state.selectedFormat = format;

  if (format === 'video') {
    elements.qualitySection.classList.remove('hidden');
  } else {
    elements.qualitySection.classList.add('hidden');
  }
}

function handleQualityChange(e) {
  state.videoQuality = e.target.value;
}

function handleConcurrentChange(e) {
  const value = parseInt(e.target.value);
  state.concurrentDownloads = value;
  elements.concurrentValue.textContent = value;
}

function handleSelectAllToggle() {
  const allSelected = state.selectedVideos.size === state.currentMetadata?.videos.length;

  // Toggle select all/deselect all
  if (allSelected) {
    // Deselect all
    state.selectedVideos.clear();
    elements.selectAllBtn.textContent = 'Select All';
  } else {
    // Select all
    state.currentMetadata?.videos.forEach((_, index) => {
      state.selectedVideos.add(index);
    });
    elements.selectAllBtn.textContent = 'Deselect All';
  }

  // Update video item checkboxes
  const videoItems = elements.videoList.querySelectorAll('.video-item');
  videoItems.forEach((item, index) => {
    const checkbox = item.querySelector('input[type="checkbox"]');
    const isSelected = state.selectedVideos.has(index);
    checkbox.checked = isSelected;
    if (isSelected) {
      item.classList.remove('excluded');
    } else {
      item.classList.add('excluded');
    }
  });

  updateDownloadButtonText();
}

function updateDownloadButtonText() {
  const totalVideos = state.currentMetadata?.videos.length || 0;
  const selectedCount = state.selectedVideos.size;
  const btn = elements.downloadAllBtn;

  if (selectedCount === 0) {
    btn.disabled = true;
  } else {
    btn.disabled = false;
  }

  if (selectedCount === totalVideos) {
    btn.innerHTML = `
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
        <polyline points="7 10 12 15 17 10"/>
        <line x1="12" y1="15" x2="12" y2="3"/>
      </svg>
      Download All
    `;
  } else if (selectedCount > 0) {
    btn.innerHTML = `
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
        <polyline points="7 10 12 15 17 10"/>
        <line x1="12" y1="15" x2="12" y2="3"/>
      </svg>
      Download Selected (${selectedCount})
    `;
  }
}

async function handleChangePath() {
  try {
    const selected = await window.__TAURI__.dialog.open({
      title: 'Select Download Folder',
      multiple: false,
      directory: true,
    });

    if (selected) {
      state.outputPath = selected;
      elements.outputPath.textContent = selected;
      showToast('Download folder updated', 'success');
    }
  } catch (error) {
    console.error('Failed to open folder dialog:', error);
    showToast('Failed to change folder', 'error');
  }
}

// ═════════════════════════════════════════════════════════════════
// DOWNLOAD PATH
// ═════════════════════════════════════════════════════════════════

async function loadDownloadPath() {
  try {
    const path = await invoke('select_download_folder');
    state.outputPath = path;
    elements.outputPath.textContent = path;
  } catch (error) {
    console.error('Failed to get download path:', error);
    elements.outputPath.textContent = 'Unknown';
  }
}

// ═════════════════════════════════════════════════════════════════
// DOWNLOAD HANDLING
// ═════════════════════════════════════════════════════════════════

async function handleDownloadAll() {
  if (!state.currentMetadata || !state.ytdlpInstalled) {
    showToast('Cannot start download', 'error');
    return;
  }

  const videos = state.currentMetadata.videos;

  // Filter to only include selected videos
  const selectedVideos = videos.filter((_, index) => state.selectedVideos.has(index));

  if (selectedVideos.length === 0) {
    showToast('No videos selected for download', 'error');
    return;
  }

  const url = state.currentUrl;
  const downloadType = state.selectedFormat;
  const outputPath = state.outputPath;

  if (selectedVideos.length === 1) {
    startSingleVideoDownload(selectedVideos[0], url, downloadType, outputPath);
  } else {
    startPlaylistDownload(url, downloadType, outputPath, selectedVideos);
  }
}

async function startSingleVideoDownload(video, url, downloadType, outputPath) {
  try {
    const downloadId = await invoke('download_video', {
      url,
      downloadType,
      outputPath,
      title: video.title,
      videoQuality: state.videoQuality,
    });

    addDownloadToUI({
      id: downloadId,
      title: video.title,
      url,
      downloadType,
      progress: 0,
      status: 'starting',
    });

    if (state.currentMetadata?.videos.length === 1) {
      showToast(`Started downloading: ${video.title}`, 'info');
    }
  } catch (error) {
    showToast(`Failed to start download: ${error}`, 'error');
  }
}

async function startPlaylistDownload(url, downloadType, outputPath, videos) {
  try {
    const concurrent = state.concurrentDownloads;
    let downloaded = 0;
    let active = 0;
    let index = 0;

    showToast(`Starting ${videos.length} videos (max ${concurrent} concurrent)`, 'info');

    const processNextBatch = async () => {
      while (index < videos.length && active < concurrent) {
        const video = videos[index++];
        active++;

        try {
          const downloadId = await invoke('download_video', {
            url: video.url,
            downloadType,
            outputPath,
            title: video.title,
            videoQuality: state.videoQuality,
          });

          downloaded++;
        } catch (error) {
          console.error(`Failed to start ${video.title}:`, error);
        } finally {
          active--;
          await new Promise(resolve => setTimeout(resolve, 500));
        }
      }
    };

    const promises = [];
    for (let i = 0; i < Math.min(concurrent, videos.length); i++) {
      promises.push(processNextBatch());
    }

    await Promise.all(promises);
    showToast(`All ${videos.length} selected downloads initiated!`, 'success');
  } catch (error) {
    showToast(`Failed to start playlist download: ${error}`, 'error');
  }
}

function addDownloadToUI(download) {
  state.activeDownloads.set(download.id, download);
  updateDownloadsList();
}

function updateDownloadProgress(id, progress, status, title, downloadType, converting) {
  // Skip updates for downloads that were cancelled by user
  // IMPORTANT: Never remove from cancelledDownloads - it's permanent to prevent race conditions
  if (state.cancelledDownloads.has(id)) {
    console.log(`Ignoring progress event for cancelled download ${id}`);
    return;
  }

  if (!state.activeDownloads.has(id)) {
    state.activeDownloads.set(id, {
      id,
      title: title || 'Unknown',
      progress,
      status,
      downloadType: downloadType || 'video',
      converting: converting || false,
    });
  } else {
    const download = state.activeDownloads.get(id);
    download.progress = progress;
    download.status = status;
    if (title) download.title = title;
    if (converting !== undefined) download.converting = converting;
  }

  updateDownloadsList();
}

function removeDownload(id) {
  state.activeDownloads.delete(id);
  updateDownloadsList();
}

function updateDownloadsList() {
  const allDownloads = Array.from(state.activeDownloads.values());

  // Filter to only show actively downloading/converting files
  const activeDownloads = allDownloads.filter(d =>
    d.status === 'downloading' ||
    d.status === 'starting' ||
    d.status === 'converting' ||
    d.status === 'download_complete'
  );

  // Update counter to show only active downloads and conversions
  state.downloadCounter = activeDownloads.length;
  elements.downloadCount.textContent = state.downloadCounter;

  // Remove completed downloads from the map after a delay
  allDownloads.forEach(download => {
    if (download.status === 'completed' || download.status === 'failed') {
      // Remove from the active map after 3 seconds
      setTimeout(() => {
        state.activeDownloads.delete(download.id);
        updateDownloadsList();
      }, 3000);
    }
  });

  // Get existing download IDs in the DOM
  const existingIds = new Set(
    Array.from(elements.downloadsList.children)
      .map(child => child.dataset.id)
      .filter(id => id && id !== 'empty-state')
  );

  // Get current download IDs
  const currentIds = new Set(activeDownloads.map(d => d.id));

  // Remove downloads that are no longer active
  existingIds.forEach(id => {
    if (!currentIds.has(id)) {
      const element = elements.downloadsList.querySelector(`[data-id="${id}"]`);
      if (element) {
        element.remove();
      }
    }
  });

  // Handle empty state
  if (activeDownloads.length === 0) {
    if (!elements.downloadsList.querySelector('.empty-state')) {
      elements.downloadsList.innerHTML = `
        <div class="empty-state" data-id="empty-state">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="17 8 12 3 7 8"/>
            <line x1="12" y1="3" x2="12" y2="15"/>
          </svg>
          <span>No active downloads</span>
        </div>
      `;
    }
    return;
  }

  // Remove empty state if it exists
  const emptyState = elements.downloadsList.querySelector('.empty-state');
  if (emptyState) {
    emptyState.remove();
  }

  // Update or create download items
  activeDownloads.forEach(download => {
    let element = elements.downloadsList.querySelector(`[data-id="${download.id}"]`);

    if (element) {
      // Update existing element in-place (no flashing)
      updateDownloadItemElement(element, download);
    } else {
      // Create new element for new download
      const downloadItem = createDownloadItem(download);
      elements.downloadsList.appendChild(downloadItem);
    }
  });
}

function createDownloadItem(download) {
  const div = document.createElement('div');
  div.className = 'download-item';
  div.dataset.id = download.id;

  const statusClass = download.status || 'downloading';
  const progress = download.progress || 0;
  const isDownloading = statusClass === 'downloading';
  const isConverting = statusClass === 'converting';
  const isDownloadComplete = statusClass === 'download_complete';
  const isCancellable = isDownloading || isConverting || isDownloadComplete;

  // Display different status text for each stage
  let statusText = statusClass;
  if (isDownloadComplete) {
    statusText = 'Download complete';
  } else if (isConverting) {
    statusText = `Converting ${Math.round(progress)}%`;
  }

  div.innerHTML = `
    <div class="download-header">
      <div class="download-info">
        <div class="download-title" title="${download.title}">${download.title}</div>
        <div class="download-type">${download.downloadType || 'video'}</div>
      </div>
      <div class="download-status ${statusClass}">${statusText}</div>
      ${isCancellable ? `
        <button class="download-cancel-btn" data-id="${download.id}">Cancel</button>
      ` : ''}
    </div>
    <div class="download-progress">
      <div class="progress-track">
        <div class="progress-bar" style="width: ${progress}%"></div>
      </div>
      <div class="progress-text">${Math.round(progress)}%</div>
    </div>
  `;

  if (isCancellable) {
    const cancelBtn = div.querySelector('.download-cancel-btn');
    cancelBtn.addEventListener('click', () => handleCancelDownload(download.id));
  }

  return div;
}

function updateDownloadItemElement(element, download) {
  const statusClass = download.status || 'downloading';
  const progress = download.progress || 0;
  const isDownloading = statusClass === 'downloading';
  const isConverting = statusClass === 'converting';
  const isDownloadComplete = statusClass === 'download_complete';
  const isCancellable = isDownloading || isConverting || isDownloadComplete;

  // Update title
  const titleElement = element.querySelector('.download-title');
  if (titleElement && download.title) {
    titleElement.textContent = download.title;
    titleElement.title = download.title;
  }

  // Update status
  const statusElement = element.querySelector('.download-status');
  if (statusElement) {
    statusElement.className = `download-status ${statusClass}`;

    // Update status text
    let statusText = statusClass;
    if (isDownloadComplete) {
      statusText = 'Download complete';
    } else if (isConverting) {
      statusText = `Converting ${Math.round(progress)}%`;
    }
    statusElement.textContent = statusText;
  }

  // Update progress bar (this is the most frequent update)
  const progressBar = element.querySelector('.progress-bar');
  if (progressBar) {
    progressBar.style.width = `${progress}%`;
  }

  // Update progress text
  const progressText = element.querySelector('.progress-text');
  if (progressText) {
    progressText.textContent = `${Math.round(progress)}%`;
  }

  // Update cancel button visibility
  let cancelBtn = element.querySelector('.download-cancel-btn');

  if (isCancellable && !cancelBtn) {
    // Add cancel button if it doesn't exist
    const header = element.querySelector('.download-header');
    cancelBtn = document.createElement('button');
    cancelBtn.className = 'download-cancel-btn';
    cancelBtn.dataset.id = download.id;
    cancelBtn.textContent = 'Cancel';
    cancelBtn.addEventListener('click', () => handleCancelDownload(download.id));
    header.appendChild(cancelBtn);
  } else if (!isCancellable && cancelBtn) {
    // Remove cancel button if it exists but shouldn't
    cancelBtn.remove();
  }
}

async function handleCancelDownload(downloadId) {
  try {
    // Mark as cancelled to prevent future progress updates from recreating it
    // IMPORTANT: Never remove from cancelledDownloads - it's permanent to prevent race conditions
    state.cancelledDownloads.add(downloadId);

    await invoke('cancel_download', { downloadId });
    removeDownload(downloadId);
    showToast('Download cancelled', 'info');
  } catch (error) {
    // If cancellation failed, remove from cancelled set so updates continue
    state.cancelledDownloads.delete(downloadId);
    showToast(`Failed to cancel download: ${error}`, 'error');
  }
}

async function handleCancelDownloads() {
  const downloads = Array.from(state.activeDownloads.keys());

  // Mark all as cancelled first
  // IMPORTANT: Never remove from cancelledDownloads - it's permanent to prevent race conditions
  downloads.forEach(id => state.cancelledDownloads.add(id));

  for (const downloadId of downloads) {
    try {
      await invoke('cancel_download', { downloadId });
    } catch (error) {
      console.error(`Failed to cancel download ${downloadId}:`, error);
    }
  }

  state.activeDownloads.clear();
  updateDownloadsList();
  showToast('All downloads cancelled', 'info');
}

// ═════════════════════════════════════════════════════════════════
// TOAST NOTIFICATIONS
// ═════════════════════════════════════════════════════════════════

function showToast(message, type = 'info') {
  const toast = document.createElement('div');
  toast.className = `toast ${type}`;
  toast.textContent = message;

  elements.toastContainer.appendChild(toast);

  setTimeout(() => {
    toast.style.animation = 'slideIn 200ms ease reverse';
    setTimeout(() => toast.remove(), 200);
  }, 3000);
}

// ═════════════════════════════════════════════════════════════════
// STARTUP
// ═════════════════════════════════════════════════════════════════

document.addEventListener('DOMContentLoaded', initialize);
