<script setup>
import { ref, onMounted } from 'vue'
import FileUploader from './components/FileUploader.vue'
import DocumentPreview from './components/DocumentPreview.vue'
import ExtractedForm from './components/ExtractedForm.vue'
import RawInspector from './components/RawInspector.vue'
import { checkHealth, scanDocument } from './services/api.js'

const backendStatus = ref('checking') // 'online', 'offline', 'checking'
const selectedFile = ref(null)
const scanning = ref(false)
const scanResult = ref(null)
const errorMessage = ref('')
const invalidDocWarning = ref('')
const activeView = ref('split') // 'split', 'preview-only', 'form-only'

// Default empty form state
const defaultFields = {
  document_type: 'Delivery Challan',
  voucher_no: '',
  date: '',
  party: '',
  company: '',
  broker: '',
  gst_no: '',
  items: [],
  rollsQty: '0 / 0.00 PCS',
  taxable: '0.00',
  cgst: '0.00',
  sgst: '0.00',
  total_amount: '0.00',
  notes: '',
}

const currentFields = ref({ ...defaultFields })

onMounted(async () => {
  const health = await checkHealth()
  if (health.status === 'healthy') {
    backendStatus.value = 'online'
  } else {
    backendStatus.value = 'offline'
  }
})

function onFileSelected(file, autoScan = false) {
  selectedFile.value = file
  errorMessage.value = ''
  invalidDocWarning.value = ''
  if (!file) {
    scanResult.value = null
    currentFields.value = { ...defaultFields }
  } else if (autoScan) {
    startScan()
  }
}

function resetToUpload() {
  selectedFile.value = null
  scanResult.value = null
  invalidDocWarning.value = ''
  errorMessage.value = ''
  currentFields.value = { ...defaultFields }
}

async function startScan() {
  if (!selectedFile.value || scanning.value) return
  scanning.value = true
  errorMessage.value = ''
  invalidDocWarning.value = ''

  try {
    const res = await scanDocument(selectedFile.value)
    if (res.success) {
      scanResult.value = res
      currentFields.value = {
        ...defaultFields,
        ...(res.fields || {}),
      }
      if (res.is_valid_document === false) {
        invalidDocWarning.value = res.message || 'The uploaded file does not appear to be an invoice or delivery challan. Please upload or scan a valid document.'
      }
    } else {
      errorMessage.value = res.message || res.error || 'Failed to scan document'
    }
  } catch (err) {
    errorMessage.value = err.message || 'Error communicating with Rust scanner backend'
  } finally {
    scanning.value = false
  }
}

function handleSave() {
  alert('Data verified successfully! Ready to submit to database or ERP.')
}
</script>

<template>
  <div class="app-shell">
    <!-- Navbar -->
    <header class="app-header">
      <div class="nav-brand">
        <div class="brand-logo">⚡</div>
        <div class="brand-info">
          <h1 class="brand-title">Document Scanner</h1>
        </div>
      </div>
    </header>

    <!-- Main Container -->
    <main class="app-main">
      <!-- Error Banner -->
      <div v-if="errorMessage" class="error-banner">
        <span>⚠️ {{ errorMessage }}</span>
        <button class="banner-close" @click="errorMessage = ''">✕</button>
      </div>

      <!-- Invalid Document Warning Banner -->
      <div v-if="invalidDocWarning" class="invalid-doc-banner">
        <div class="banner-left">
          <span class="banner-icon">⚠️</span>
          <div>
            <div class="banner-title">Invalid or Non-Business Document Detected</div>
            <div class="banner-text">{{ invalidDocWarning }}</div>
          </div>
        </div>
        <div class="banner-actions">
          <button type="button" class="btn-banner-action" @click="resetToUpload">Upload Valid Document</button>
          <button type="button" class="banner-close" @click="invalidDocWarning = ''">✕</button>
        </div>
      </div>

      <!-- Upload Bar -->
      <section class="upload-section">
        <FileUploader
          :loading="scanning"
          @file-selected="onFileSelected"
          @scan="startScan"
        />
      </section>

      <!-- Workspace Area -->
      <section class="workspace-grid">
        <!-- Left: Document Preview -->
        <div class="col-preview">
          <DocumentPreview :file="selectedFile" />
        </div>

        <!-- Right: Extracted Form -->
        <div class="col-form">
          <ExtractedForm
            v-model="currentFields"
            :confidence="scanResult?.confidence || 0"
            :processing-time-ms="scanResult?.processing_time_ms || 0"
            :source-type="scanResult?.source_type || 'image'"
            :busy="scanning"
            @save="handleSave"
          />
        </div>
      </section>

      <!-- Bottom Inspector: Raw Text & JSON -->
      <section class="inspector-section">
        <RawInspector
          :raw-text="scanResult?.raw_text || ''"
          :raw-response="scanResult || {}"
        />
      </section>
    </main>
  </div>
</template>

<style scoped>
.app-shell {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}
.app-header {
  height: 64px;
  background: #090f17;
  border-bottom: 1px solid var(--border);
  padding: 0 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  position: sticky;
  top: 0;
  z-index: 50;
}
.nav-brand {
  display: flex;
  align-items: center;
  gap: 12px;
}
.brand-logo {
  width: 36px;
  height: 36px;
  background: linear-gradient(135deg, #3b82f6, #1d4ed8);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  box-shadow: 0 0 15px var(--primary-glow);
}
.brand-title {
  font-size: 16px;
  font-weight: 700;
  color: #ffffff;
  letter-spacing: -0.2px;
}
.brand-badge {
  font-size: 11px;
  color: var(--text-muted);
}
.nav-controls {
  display: flex;
  align-items: center;
  gap: 14px;
}
.btn-demo {
  background: rgba(59, 130, 246, 0.1);
  border: 1px solid rgba(59, 130, 246, 0.3);
  color: #60a5fa;
  font-size: 12px;
  font-weight: 600;
  padding: 6px 14px;
  border-radius: 6px;
  transition: all 0.15s;
}
.btn-demo:hover {
  background: rgba(59, 130, 246, 0.2);
  transform: translateY(-1px);
}
.status-pill {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  padding: 6px 12px;
  border-radius: 20px;
  border: 1px solid transparent;
}
.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}
.status-online {
  background: var(--success-bg);
  color: var(--success);
  border-color: rgba(16, 185, 129, 0.3);
}
.status-online .status-dot {
  background: var(--success);
  box-shadow: 0 0 8px var(--success);
}
.status-offline {
  background: var(--danger-bg);
  color: var(--danger);
  border-color: rgba(239, 68, 68, 0.3);
}
.status-offline .status-dot {
  background: var(--danger);
}
.app-main {
  flex: 1;
  max-width: 1680px;
  width: 100%;
  margin: 0 auto;
  padding: 20px 24px 60px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.error-banner {
  background: var(--danger-bg);
  border: 1px solid rgba(239, 68, 68, 0.4);
  color: #fca5a5;
  padding: 12px 18px;
  border-radius: var(--radius);
  font-size: 13px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.invalid-doc-banner {
  background: rgba(245, 158, 11, 0.12);
  border: 1px solid rgba(245, 158, 11, 0.45);
  color: #fde68a;
  padding: 14px 20px;
  border-radius: var(--radius-lg);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  box-shadow: 0 4px 18px rgba(245, 158, 11, 0.15);
}
.banner-left {
  display: flex;
  align-items: center;
  gap: 14px;
}
.banner-icon {
  font-size: 26px;
  line-height: 1;
}
.banner-title {
  font-weight: 700;
  font-size: 14px;
  color: #fbbf24;
  margin-bottom: 2px;
}
.banner-text {
  color: #fef3c7;
  font-size: 13px;
  line-height: 1.4;
}
.banner-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}
.btn-banner-action {
  background: #f59e0b;
  color: #0c141f;
  border: none;
  font-weight: 700;
  font-size: 12px;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s ease;
}
.btn-banner-action:hover {
  background: #d97706;
}
.banner-close {
  background: transparent;
  border: none;
  color: inherit;
  font-size: 16px;
  padding: 0 4px;
  cursor: pointer;
}
.upload-section {
  width: 100%;
}
.workspace-grid {
  display: grid;
  grid-template-columns: 460px 1fr;
  gap: 20px;
  align-items: start;
}
.col-preview {
  position: sticky;
  top: 84px;
  max-height: calc(100vh - 120px);
}
.col-form {
  min-width: 0;
}
.inspector-section {
  width: 100%;
  margin-top: 10px;
}

@media (max-width: 1100px) {
  .workspace-grid {
    grid-template-columns: 1fr;
  }
  .col-preview {
    position: static !important;
    top: auto !important;
    max-height: none !important;
  }
}
@media (max-width: 768px) {
  .app-main {
    padding: 12px 12px 40px;
    gap: 14px;
  }
  .col-preview {
    position: static !important;
    top: auto !important;
    max-height: none !important;
  }
}
</style>
