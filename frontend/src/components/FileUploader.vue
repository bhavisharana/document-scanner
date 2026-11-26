<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import CameraScanner from './CameraScanner.vue'

const props = defineProps({
  loading: { type: Boolean, default: false },
})

const emit = defineEmits(['file-selected', 'scan'])

const fileInput = ref(null)
const mobileCameraInput = ref(null)
const currentFile = ref(null)
const isDragging = ref(false)
const showCameraModal = ref(false)
const isMobile = ref(false)

onMounted(() => {
  window.addEventListener('paste', handlePaste)
  isMobile.value = /Android|iPhone|iPad|iPod/i.test(navigator.userAgent) || 
    (window.matchMedia && window.matchMedia('(max-width: 800px) and (pointer: coarse)').matches)
})

onUnmounted(() => {
  window.removeEventListener('paste', handlePaste)
})

function handlePaste(e) {
  if (props.loading) return
  const items = e.clipboardData?.items
  if (!items) return

  for (let i = 0; i < items.length; i++) {
    if (items[i].type.startsWith('image/')) {
      const blob = items[i].getAsFile()
      if (blob) {
        const file = new File([blob], `pasted_document_${Date.now()}.png`, { type: blob.type })
        handleFile(file, true)
        break
      }
    }
  }
}

function onDragOver(e) {
  e.preventDefault()
  isDragging.value = true
}

function onDragLeave() {
  isDragging.value = false
}

function onDrop(e) {
  e.preventDefault()
  isDragging.value = false
  if (e.dataTransfer.files && e.dataTransfer.files[0]) {
    handleFile(e.dataTransfer.files[0], false)
  }
}

function onFileChange(e) {
  if (e.target.files && e.target.files[0]) {
    handleFile(e.target.files[0], false)
  }
}

function onMobileCameraChange(e) {
  if (e.target.files && e.target.files[0]) {
    handleFile(e.target.files[0], true) // Auto-scan immediately on mobile photo capture
  }
}

function handleFile(file, autoScan = false) {
  const ext = file.name.split('.').pop().toLowerCase()
  if (!['pdf', 'png', 'jpg', 'jpeg', 'webp', 'tiff', 'bmp'].includes(ext)) {
    alert('Please upload a PDF or Image (PNG, JPG, JPEG, WEBP, TIFF).')
    return
  }
  currentFile.value = file
  emit('file-selected', file, autoScan)
}

function triggerBrowse() {
  fileInput.value.click()
}

function triggerCamera() {
  // If on mobile or in an insecure HTTP context where WebRTC is restricted, open native phone camera
  const isSecure = window.isSecureContext || window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1'
  if (isMobile.value || !isSecure) {
    if (mobileCameraInput.value) {
      mobileCameraInput.value.click()
      return
    }
  }
  // Otherwise open desktop/webcam viewfinder modal
  showCameraModal.value = true
}

function onCameraCapture(file) {
  showCameraModal.value = false
  currentFile.value = file
  // Auto-scan immediately upon capture
  emit('file-selected', file, true)
}

function clear() {
  currentFile.value = null
  if (fileInput.value) fileInput.value.value = ''
  if (mobileCameraInput.value) mobileCameraInput.value.value = ''
  emit('file-selected', null)
}

function formatSize(bytes) {
  if (!bytes) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}
</script>

<template>
  <div class="uploader-container">
    <!-- Standard File Browse Input -->
    <input
      ref="fileInput"
      type="file"
      accept=".pdf,.png,.jpg,.jpeg,.webp,.tiff,.bmp"
      class="hidden-input"
      @change="onFileChange"
    />

    <!-- Native Mobile Camera Input (Works on all mobile devices even over HTTP) -->
    <input
      ref="mobileCameraInput"
      type="file"
      accept="image/*"
      capture="environment"
      class="hidden-input"
      @change="onMobileCameraChange"
    />

    <!-- When No File is Loaded -->
    <div v-if="!currentFile" class="options-grid">
      <!-- Option 1: Camera Scanner -->
      <div class="action-card card-camera" @click="triggerCamera">
        <div class="card-glow"></div>
        <div class="card-icon-circle icon-camera">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/>
            <circle cx="12" cy="13" r="4"/>
          </svg>
        </div>
        <div class="card-content">
          <div class="card-badge">RECOMMENDED</div>
          <h3 class="card-title">{{ isMobile ? 'Take Photo & Scan' : 'Scan with Camera' }}</h3>
          <p class="card-desc">
            {{ isMobile ? 'Opens phone camera to snap document and auto-fill the form instantly' : 'Use live camera/webcam to snap and auto-fill form instantly without saving files' }}
          </p>
        </div>
        <button type="button" class="btn-primary-action">
          <span>📷 {{ isMobile ? 'Snap Photo & Scan' : 'Open Camera Scanner' }}</span>
        </button>
      </div>

      <!-- Option 2: Upload File or Drag & Drop -->
      <div
        class="action-card card-upload"
        :class="{ 'dropzone-active': isDragging }"
        @dragover="onDragOver"
        @dragleave="onDragLeave"
        @drop="onDrop"
        @click="triggerBrowse"
      >
        <div class="card-icon-circle icon-upload">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="17 8 12 3 7 8"/>
            <line x1="12" y1="3" x2="12" y2="15"/>
          </svg>
        </div>
        <div class="card-content">
          <h3 class="card-title">Upload Document</h3>
          <p class="card-desc">Click to browse or drag & drop PDF, JPG, PNG (up to 25MB) · Or press Ctrl+V to paste</p>
        </div>
        <button type="button" class="btn-secondary-action">
          <span>📁 Browse Files</span>
        </button>
      </div>
    </div>

    <!-- When File is Loaded -->
    <div v-else class="file-loaded-card">
      <div class="file-info">
        <div class="file-icon-box" :class="currentFile.name.toLowerCase().endsWith('.pdf') ? 'icon-pdf' : 'icon-img'">
          {{ currentFile.name.split('.').pop().toUpperCase() }}
        </div>
        <div class="file-meta">
          <p class="file-name" :title="currentFile.name">{{ currentFile.name }}</p>
          <span class="file-size">{{ formatSize(currentFile.size) }}</span>
        </div>
      </div>

      <div class="file-actions">
        <button
          type="button"
          class="btn-camera-rescan"
          title="Scan another document with camera"
          :disabled="loading"
          @click="triggerCamera"
        >
          📷 Scan via Camera
        </button>
        <button
          type="button"
          class="btn-clear"
          title="Remove file"
          :disabled="loading"
          @click.stop="clear"
        >
          ✕ Change
        </button>
        <button
          type="button"
          class="btn-scan"
          :disabled="loading"
          @click="emit('scan')"
        >
          <span v-if="loading" class="spinner"></span>
          <span>{{ loading ? 'Scanning with Rust...' : '⚡ Scan Document' }}</span>
        </button>
      </div>
    </div>

    <!-- Camera Scanner Modal -->
    <CameraScanner
      v-if="showCameraModal"
      :is-open="showCameraModal"
      @close="showCameraModal = false"
      @capture="onCameraCapture"
    />
  </div>
</template>

<style scoped>
.uploader-container {
  width: 100%;
}
.hidden-input {
  display: none;
}

.options-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

@media (max-width: 768px) {
  .options-grid {
    grid-template-columns: 1fr;
  }
}

.action-card {
  position: relative;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 24px 22px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 12px;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  overflow: hidden;
}

.action-card:hover {
  transform: translateY(-2px);
  border-color: var(--primary);
  background: var(--bg-card-hover);
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.3), 0 0 20px var(--primary-glow);
}

.card-camera {
  border-color: rgba(59, 130, 246, 0.4);
  background: linear-gradient(180deg, rgba(59, 130, 246, 0.05) 0%, rgba(17, 26, 38, 0.95) 100%);
}

.card-camera:hover {
  border-color: #3b82f6;
  box-shadow: 0 10px 30px rgba(59, 130, 246, 0.25);
}

.card-badge {
  display: inline-block;
  background: rgba(16, 185, 129, 0.15);
  color: var(--success);
  border: 1px solid rgba(16, 185, 129, 0.3);
  font-size: 10px;
  font-weight: 800;
  padding: 2px 8px;
  border-radius: 10px;
  letter-spacing: 0.5px;
  margin-bottom: 4px;
}

.card-icon-circle {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.2s;
}

.action-card:hover .card-icon-circle {
  transform: scale(1.08);
}

.icon-camera {
  background: rgba(59, 130, 246, 0.15);
  color: #60a5fa;
  border: 1px solid rgba(59, 130, 246, 0.3);
}

.icon-upload {
  background: rgba(148, 163, 184, 0.1);
  color: #94a3b8;
  border: 1px solid rgba(148, 163, 184, 0.2);
}

.card-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.card-title {
  font-size: 15px;
  font-weight: 700;
  color: var(--text-main);
}

.card-desc {
  font-size: 12px;
  color: var(--text-muted);
  max-width: 320px;
  line-height: 1.4;
}

.btn-primary-action {
  margin-top: 4px;
  background: linear-gradient(135deg, #2563eb, #1d4ed8);
  border: 1px solid #3b82f6;
  color: #ffffff;
  padding: 8px 18px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
  box-shadow: 0 4px 12px var(--primary-glow);
  transition: all 0.15s;
}

.btn-primary-action:hover {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
}

.btn-secondary-action {
  margin-top: 4px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--border);
  color: var(--text-main);
  padding: 8px 18px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  transition: all 0.15s;
}

.btn-secondary-action:hover {
  background: rgba(255, 255, 255, 0.1);
}

.dropzone-active {
  border-color: var(--primary) !important;
  background: var(--bg-card-hover) !important;
  box-shadow: 0 0 20px var(--primary-glow) !important;
}

.file-loaded-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  gap: 16px;
  flex-wrap: wrap;
}

.file-info {
  display: flex;
  align-items: center;
  gap: 14px;
  min-width: 0;
  flex: 1;
}

.file-icon-box {
  width: 44px;
  height: 44px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.5px;
  flex-shrink: 0;
}

.icon-pdf {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.icon-img {
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
  border: 1px solid rgba(16, 185, 129, 0.3);
}

.file-meta {
  min-width: 0;
}

.file-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-main);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-size {
  font-size: 12px;
  color: var(--text-muted);
}

.file-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.btn-camera-rescan {
  background: rgba(59, 130, 246, 0.1);
  border: 1px solid rgba(59, 130, 246, 0.3);
  color: #60a5fa;
  padding: 8px 14px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  transition: all 0.15s;
}

.btn-camera-rescan:hover:not(:disabled) {
  background: rgba(59, 130, 246, 0.2);
}

.btn-clear {
  background: transparent;
  border: 1px solid var(--border);
  color: var(--text-muted);
  padding: 8px 14px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  transition: all 0.2s;
}

.btn-clear:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.1);
  color: var(--danger);
  border-color: rgba(239, 68, 68, 0.3);
}

.btn-scan {
  background: var(--primary);
  border: none;
  color: #ffffff;
  padding: 9px 18px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.2s;
  box-shadow: 0 4px 12px var(--primary-glow);
}

.btn-scan:hover:not(:disabled) {
  background: var(--primary-hover);
  transform: translateY(-1px);
}

.btn-scan:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #ffffff;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
