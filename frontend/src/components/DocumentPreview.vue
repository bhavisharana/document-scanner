<script setup>
import { ref, computed, watch, onUnmounted } from 'vue'

const props = defineProps({
  file: { type: Object, default: null },
})

const objectUrl = ref(null)
const zoomLevel = ref(1)
const isCollapsed = ref(false)

function toggleCollapse() {
  isCollapsed.value = !isCollapsed.value
}

watch(
  () => props.file,
  (newFile) => {
    if (objectUrl.value) {
      URL.revokeObjectURL(objectUrl.value)
      objectUrl.value = null
    }
    if (newFile) {
      objectUrl.value = URL.createObjectURL(newFile)
      zoomLevel.value = 1
      isCollapsed.value = false
    }
  },
  { immediate: true }
)

onUnmounted(() => {
  if (objectUrl.value) {
    URL.revokeObjectURL(objectUrl.value)
  }
})

const isPdf = computed(() => {
  return props.file?.name?.toLowerCase().endsWith('.pdf') || props.file?.type === 'application/pdf'
})

function zoomIn() {
  if (zoomLevel.value < 2.5) zoomLevel.value = parseFloat((zoomLevel.value + 0.25).toFixed(2))
}

function zoomOut() {
  if (zoomLevel.value > 0.5) zoomLevel.value = parseFloat((zoomLevel.value - 0.25).toFixed(2))
}

function resetZoom() {
  zoomLevel.value = 1
}
</script>

<template>
  <div class="preview-container" :class="{ 'is-collapsed': isCollapsed }">
    <div class="preview-header" @click="toggleCollapse">
      <div class="preview-title">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
          <polyline points="14 2 14 8 20 8"></polyline>
        </svg>
        <span>Document Preview</span>
      </div>

      <div class="header-right" @click.stop>
        <div v-if="!isPdf && objectUrl && !isCollapsed" class="zoom-controls">
          <button class="zoom-btn" title="Zoom Out" @click="zoomOut">-</button>
          <span class="zoom-text">{{ Math.round(zoomLevel * 100) }}%</span>
          <button class="zoom-btn" title="Zoom In" @click="zoomIn">+</button>
          <button class="zoom-btn zoom-reset" title="Reset Zoom" @click="resetZoom">Fit</button>
        </div>

        <button 
          v-if="objectUrl"
          type="button"
          class="btn-toggle-collapse"
          :title="isCollapsed ? 'Expand Preview' : 'Collapse Preview'"
          @click="toggleCollapse"
        >
          <span class="collapse-icon">{{ isCollapsed ? '▼' : '▲' }}</span>
          <span class="collapse-label">{{ isCollapsed ? 'Expand' : 'Collapse' }}</span>
        </button>
      </div>
    </div>

    <div v-show="!isCollapsed" class="preview-body">
      <div v-if="!objectUrl" class="no-preview">
        <div class="no-preview-icon">📄</div>
        <p>No document selected</p>
        <span>Upload a document above to inspect and verify scanned fields</span>
      </div>

      <!-- PDF Preview -->
      <iframe
        v-else-if="isPdf"
        :src="objectUrl"
        class="pdf-frame"
        title="PDF Preview"
      ></iframe>

      <!-- Image Preview -->
      <div v-else class="image-viewport">
        <img
          :src="objectUrl"
          :style="{ transform: `scale(${zoomLevel})` }"
          class="preview-img"
          alt="Document preview"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.preview-container {
  display: flex;
  flex-direction: column;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  height: 100%;
  min-height: 600px;
  overflow: hidden;
  transition: min-height 0.2s ease, height 0.2s ease;
}
.preview-container.is-collapsed {
  min-height: auto !important;
  height: auto !important;
}
.preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 18px;
  background: #0d1521;
  border-bottom: 1px solid var(--border);
  user-select: none;
}
.preview-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-main);
}
.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}
.btn-toggle-collapse {
  background: var(--bg-app);
  border: 1px solid var(--border);
  color: #60a5fa;
  font-size: 11px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  gap: 5px;
  cursor: pointer;
  transition: all 0.15s;
}
.btn-toggle-collapse:hover {
  background: rgba(59, 130, 246, 0.15);
  border-color: var(--primary);
  color: #93c5fd;
}
.zoom-controls {
  display: flex;
  align-items: center;
  gap: 6px;
}
.zoom-btn {
  background: var(--bg-app);
  border: 1px solid var(--border);
  color: var(--text-muted);
  width: 26px;
  height: 26px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: 600;
  transition: all 0.15s;
}
.zoom-btn:hover {
  color: var(--text-main);
  border-color: var(--primary);
}
.zoom-reset {
  width: auto;
  padding: 0 8px;
  font-size: 11px;
}
.zoom-text {
  font-size: 11px;
  font-family: var(--font-mono);
  color: var(--text-muted);
  min-width: 40px;
  text-align: center;
}
.preview-body {
  flex: 1;
  position: relative;
  background: #070b12;
  overflow: auto;
  display: flex;
  align-items: center;
  justify-content: center;
}
.no-preview {
  text-align: center;
  color: var(--text-dim);
  padding: 40px;
}
.no-preview-icon {
  font-size: 48px;
  margin-bottom: 12px;
  opacity: 0.5;
}
.no-preview p {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-muted);
  margin-bottom: 4px;
}
.no-preview span {
  font-size: 12px;
}
.pdf-frame {
  width: 100%;
  height: 100%;
  border: none;
}
.image-viewport {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  overflow: auto;
  padding: 20px;
}
.preview-img {
  max-width: 100%;
  transform-origin: top center;
  transition: transform 0.15s ease-out;
  border-radius: 4px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
}

@media (max-width: 1100px) {
  .preview-container {
    min-height: auto;
    height: 420px;
  }
}
@media (max-width: 768px) {
  .preview-container {
    min-height: auto;
    height: 340px;
  }
  .image-viewport {
    padding: 10px;
  }
  .preview-header {
    padding: 10px 14px;
  }
  .collapse-label {
    display: none;
  }
}
</style>
