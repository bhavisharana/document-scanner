<script setup>
import { ref, onMounted, onUnmounted } from 'vue'

const props = defineProps({
  isOpen: { type: Boolean, default: false },
})

const emit = defineEmits(['close', 'capture'])

const videoRef = ref(null)
const canvasRef = ref(null)
const stream = ref(null)
const devices = ref([])
const selectedDeviceId = ref('')
const error = ref('')
const isStarting = ref(false)
const isCapturing = ref(false)
const showFlash = ref(false)
const mobileFallbackInput = ref(null)

function onMobileFallback(e) {
  if (e.target.files && e.target.files[0]) {
    stopCamera()
    emit('capture', e.target.files[0])
  }
}

onMounted(async () => {
  if (props.isOpen) {
    await startCamera()
  }
})

onUnmounted(() => {
  stopCamera()
})

async function getAvailableCameras() {
  try {
    const allDevices = await navigator.mediaDevices.enumerateDevices()
    devices.value = allDevices.filter(d => d.kind === 'videoinput')
    if (devices.value.length > 0 && !selectedDeviceId.value) {
      // Prefer back/environment camera if available
      const backCam = devices.value.find(d => 
        d.label.toLowerCase().includes('back') || 
        d.label.toLowerCase().includes('rear') ||
        d.label.toLowerCase().includes('environment')
      )
      selectedDeviceId.value = backCam ? backCam.deviceId : devices.value[0].deviceId
    }
  } catch (err) {
    console.warn('Could not enumerate video devices:', err)
  }
}

async function startCamera() {
  error.value = ''
  isStarting.value = true

  if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
    error.value = 'Camera access is not supported by your browser or environment (requires HTTPS or localhost).'
    isStarting.value = false
    return
  }

  stopCamera()

  try {
    const videoConstraints = {
      width: { ideal: 1920, min: 1280 },
      height: { ideal: 1080, min: 720 },
    }

    if (selectedDeviceId.value) {
      videoConstraints.deviceId = { exact: selectedDeviceId.value }
    } else {
      videoConstraints.facingMode = { ideal: 'environment' }
    }

    const mediaStream = await navigator.mediaDevices.getUserMedia({
      video: videoConstraints,
      audio: false,
    })

    stream.value = mediaStream

    if (videoRef.value) {
      videoRef.value.srcObject = mediaStream
      await videoRef.value.play().catch(e => console.warn('Video play warning:', e))
    }

    await getAvailableCameras()
  } catch (err) {
    console.error('Camera startup error:', err)
    if (err.name === 'NotAllowedError' || err.name === 'PermissionDeniedError') {
      error.value = 'Camera permission denied. Please allow camera access in your browser settings to scan documents.'
    } else if (err.name === 'NotFoundError' || err.name === 'DevicesNotFoundError') {
      error.value = 'No camera device found on your system. Please connect a webcam or use file upload.'
    } else if (err.name === 'NotReadableError' || err.name === 'TrackStartError') {
      error.value = 'Camera is currently in use by another application. Please close other camera apps and retry.'
    } else {
      error.value = `Unable to start camera: ${err.message || err.name}`
    }
  } finally {
    isStarting.value = false
  }
}

function stopCamera() {
  if (stream.value) {
    stream.value.getTracks().forEach(track => track.stop())
    stream.value = null
  }
  if (videoRef.value) {
    videoRef.value.srcObject = null
  }
}

async function onDeviceChange() {
  await startCamera()
}

async function capture() {
  if (!videoRef.value || !stream.value) return
  isCapturing.value = true
  showFlash.value = true

  setTimeout(() => {
    showFlash.value = false
  }, 180)

  const video = videoRef.value
  const canvas = canvasRef.value || document.createElement('canvas')
  canvas.width = video.videoWidth || 1920
  canvas.height = video.videoHeight || 1080

  const ctx = canvas.getContext('2d')
  ctx.drawImage(video, 0, 0, canvas.width, canvas.height)

  canvas.toBlob((blob) => {
    isCapturing.value = false
    if (!blob) {
      error.value = 'Failed to capture image frame.'
      return
    }

    const filename = `camera_scan_${Date.now()}.jpg`
    const file = new File([blob], filename, { type: 'image/jpeg' })

    stopCamera()
    emit('capture', file)
  }, 'image/jpeg', 0.95)
}

function close() {
  stopCamera()
  emit('close')
}
</script>

<template>
  <div class="camera-modal-backdrop" @click.self="close">
    <div class="camera-modal-card">
      <!-- Modal Header -->
      <div class="modal-header">
        <div class="modal-title">
          <span class="camera-badge">LIVE SCAN</span>
          <h3>Document Camera Scanner</h3>
        </div>
        <div class="header-controls">
          <select
            v-if="devices.length > 1"
            v-model="selectedDeviceId"
            class="device-select"
            @change="onDeviceChange"
          >
            <option
              v-for="(dev, idx) in devices"
              :key="dev.deviceId || idx"
              :value="dev.deviceId"
            >
              {{ dev.label || `Camera ${idx + 1}` }}
            </option>
          </select>

          <button type="button" class="btn-close" @click="close" title="Close camera">✕</button>
        </div>
      </div>

      <!-- Viewfinder Area -->
      <div class="viewfinder-container">
        <!-- Native Mobile Fallback File Input -->
        <input
          ref="mobileFallbackInput"
          type="file"
          accept="image/*"
          capture="environment"
          class="hidden-input"
          @change="onMobileFallback"
        />

        <!-- Error State -->
        <div v-if="error" class="camera-error">
          <div class="error-icon">📷⚠️</div>
          <h4>Live Viewfinder Inactive</h4>
          <p>{{ error }}</p>
          <div class="error-actions">
            <button type="button" class="btn-snap-mobile" @click="mobileFallbackInput?.click()">
              📱 Snap Photo with Phone Camera
            </button>
            <button type="button" class="btn-retry" @click="startCamera">🔄 Try Live Feed</button>
            <button type="button" class="btn-cancel" @click="close">Cancel</button>
          </div>
        </div>

        <!-- Starting Loader -->
        <div v-else-if="isStarting" class="camera-loading">
          <div class="scanner-spinner"></div>
          <p>Initializing camera...</p>
        </div>

        <!-- Live Video -->
        <div v-show="!error && !isStarting" class="video-wrapper">
          <video
            ref="videoRef"
            autoplay
            playsinline
            muted
            class="camera-video"
          ></video>

          <!-- Visual Flash Overlay -->
          <div v-if="showFlash" class="shutter-flash"></div>

          <!-- Document Framing Overlay -->
          <div class="overlay-mask">
            <div class="frame-guide">
              <span class="corner tl"></span>
              <span class="corner tr"></span>
              <span class="corner bl"></span>
              <span class="corner br"></span>
              <div class="scanning-laser"></div>
              <div class="guide-text">
                <span>📄 Align document within frame & hold steady</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="modal-footer">
        <div class="footer-tip">
          <span>💡 Hold document flat with good lighting for best OCR results</span>
        </div>

        <div class="footer-actions">
          <button type="button" class="btn-secondary" @click="close">Cancel</button>
          <button
            type="button"
            class="btn-capture"
            :disabled="isStarting || !!error || isCapturing"
            @click="capture"
          >
            <span class="shutter-ring">
              <span class="shutter-circle"></span>
            </span>
            <span>{{ isCapturing ? 'Capturing...' : '📸 Capture & Auto-Scan' }}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.camera-modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background: rgba(3, 7, 18, 0.85);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  animation: fadeIn 0.2s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.camera-modal-card {
  width: 100%;
  max-width: 860px;
  background: #0d1522;
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.7), 0 0 30px var(--primary-glow);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: slideUp 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.modal-header {
  padding: 16px 22px;
  background: #090e17;
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.modal-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.modal-title h3 {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-main);
  letter-spacing: -0.2px;
}

.camera-badge {
  background: rgba(16, 185, 129, 0.15);
  color: var(--success);
  border: 1px solid rgba(16, 185, 129, 0.3);
  font-size: 10px;
  font-weight: 800;
  padding: 3px 8px;
  border-radius: 12px;
  letter-spacing: 0.5px;
}

.header-controls {
  display: flex;
  align-items: center;
  gap: 12px;
}

.device-select {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-main);
  font-size: 12px;
  padding: 6px 12px;
  border-radius: 6px;
  outline: none;
  max-width: 200px;
}

.device-select:focus {
  border-color: var(--primary);
}

.btn-close {
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 18px;
  padding: 4px 8px;
  border-radius: 6px;
  transition: all 0.15s;
}

.btn-close:hover {
  background: rgba(239, 68, 68, 0.15);
  color: var(--danger);
}

.viewfinder-container {
  position: relative;
  width: 100%;
  height: 480px;
  background: #05080f;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.video-wrapper {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.camera-video {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.shutter-flash {
  position: absolute;
  inset: 0;
  background: white;
  z-index: 50;
  animation: flash 0.18s ease-out forwards;
}

@keyframes flash {
  0% { opacity: 0.85; }
  100% { opacity: 0; }
}

.overlay-mask {
  position: absolute;
  inset: 0;
  pointer-events: none;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.35);
}

.frame-guide {
  position: relative;
  width: 78%;
  height: 84%;
  border: 1px dashed rgba(255, 255, 255, 0.25);
  border-radius: 12px;
  box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.45);
}

.corner {
  position: absolute;
  width: 24px;
  height: 24px;
  border-color: #3b82f6;
  border-style: solid;
}

.corner.tl {
  top: -2px;
  left: -2px;
  border-width: 3px 0 0 3px;
  border-top-left-radius: 10px;
}

.corner.tr {
  top: -2px;
  right: -2px;
  border-width: 3px 3px 0 0;
  border-top-right-radius: 10px;
}

.corner.bl {
  bottom: -2px;
  left: -2px;
  border-width: 0 0 3px 3px;
  border-bottom-left-radius: 10px;
}

.corner.br {
  bottom: -2px;
  right: -2px;
  border-width: 0 3px 3px 0;
  border-bottom-right-radius: 10px;
}

.scanning-laser {
  position: absolute;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, transparent, #3b82f6, #60a5fa, transparent);
  box-shadow: 0 0 12px #3b82f6;
  animation: laserScan 2.4s ease-in-out infinite alternate;
}

@keyframes laserScan {
  0% { top: 4%; opacity: 0.2; }
  50% { opacity: 0.9; }
  100% { top: 96%; opacity: 0.2; }
}

.guide-text {
  position: absolute;
  bottom: -32px;
  left: 0;
  right: 0;
  text-align: center;
  font-size: 12px;
  color: #f1f5f9;
  font-weight: 500;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.8);
}

.camera-error {
  text-align: center;
  padding: 30px;
  max-width: 480px;
}

.error-icon {
  font-size: 44px;
  margin-bottom: 12px;
}

.camera-error h4 {
  font-size: 17px;
  color: #f87171;
  margin-bottom: 8px;
}

.camera-error p {
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.5;
  margin-bottom: 20px;
}

.error-actions {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.btn-snap-mobile {
  background: linear-gradient(135deg, #10b981, #059669);
  color: #ffffff;
  border: none;
  padding: 10px 18px;
  border-radius: 6px;
  font-weight: 700;
  font-size: 13px;
  cursor: pointer;
  box-shadow: 0 4px 14px rgba(16, 185, 129, 0.3);
  transition: all 0.15s;
}

.btn-snap-mobile:hover {
  background: linear-gradient(135deg, #34d399, #10b981);
  transform: translateY(-1px);
}

.btn-retry {
  background: var(--primary);
  color: white;
  border: none;
  padding: 9px 18px;
  border-radius: 6px;
  font-weight: 600;
  font-size: 13px;
  cursor: pointer;
}

.btn-cancel {
  background: transparent;
  border: 1px solid var(--border);
  color: var(--text-muted);
  padding: 9px 16px;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
}

.camera-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  color: var(--text-muted);
  font-size: 14px;
}

.scanner-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid rgba(59, 130, 246, 0.2);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.modal-footer {
  padding: 16px 24px;
  background: #090e17;
  border-top: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.footer-tip {
  font-size: 12px;
  color: var(--text-dim);
}

.footer-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.btn-secondary {
  background: transparent;
  border: 1px solid var(--border);
  color: var(--text-muted);
  padding: 10px 18px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  transition: all 0.15s;
}

.btn-secondary:hover {
  background: var(--bg-card);
  color: var(--text-main);
}

.btn-capture {
  background: linear-gradient(135deg, #2563eb, #1d4ed8);
  border: 1px solid #3b82f6;
  color: #ffffff;
  padding: 10px 22px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 700;
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  box-shadow: 0 4px 16px var(--primary-glow);
  transition: all 0.2s;
}

.btn-capture:hover:not(:disabled) {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(59, 130, 246, 0.4);
}

.btn-capture:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.shutter-ring {
  width: 18px;
  height: 18px;
  border: 2px solid rgba(255, 255, 255, 0.8);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.shutter-circle {
  width: 8px;
  height: 8px;
  background: white;
  border-radius: 50%;
}
</style>
