<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'

const props = defineProps({
  rawText: { type: String, default: '' },
  rawResponse: { type: Object, default: () => ({}) },
})

const activeTab = ref('text') // 'text' or 'json'

// Toast notification state
const toastMessage = ref('')
const toastTitle = ref('')
let toastTimer = null

function showToast(title, message) {
  if (toastTimer) clearTimeout(toastTimer)
  toastTitle.value = title
  toastMessage.value = message
  toastTimer = setTimeout(() => {
    toastMessage.value = ''
    toastTitle.value = ''
  }, 2400)
}

// Fallback clipboard write for environments without Clipboard API
function fallbackCopy(text) {
  const textArea = document.createElement('textarea')
  textArea.value = text
  textArea.style.position = 'fixed'
  textArea.style.opacity = '0'
  document.body.appendChild(textArea)
  textArea.focus()
  textArea.select()
  try {
    document.execCommand('copy')
  } catch (err) {
    console.error('Fallback copy failed', err)
  }
  document.body.removeChild(textArea)
}

function truncate(str, max = 28) {
  if (!str) return ''
  return str.length > max ? str.slice(0, max) + '…' : str
}

// Main copy function
async function copyToClipboard(content, title = 'Copied to clipboard!', preview = '') {
  if (!content) return
  try {
    if (navigator?.clipboard?.writeText) {
      await navigator.clipboard.writeText(content)
    } else {
      fallbackCopy(content)
    }
    showToast(title, preview || truncate(content, 35))
  } catch {
    fallbackCopy(content)
    showToast(title, preview || truncate(content, 35))
  }
}

// Sentence detection within a line
function getSentenceForToken(line, tokenStartIndex) {
  if (!line || !line.trim()) return ''
  const trimmedLine = line.trim()

  // Match sentences ending in '.', '!', '?' followed by whitespace or line end.
  // Avoids breaking on decimals like 125.50 or item numbers.
  const sentenceRegex = /[^.!?\n]+(?:[.!?]+(?=\s|$)|$)/g
  let match
  let matchedSentence = ''

  while ((match = sentenceRegex.exec(line)) !== null) {
    const start = match.index
    const end = start + match[0].length
    if (tokenStartIndex >= start && tokenStartIndex <= end) {
      matchedSentence = match[0].trim()
      break
    }
  }

  return matchedSentence || trimmedLine
}

// Pre-parse lines into tokens (words vs whitespace)
const parsedLines = computed(() => {
  if (!props.rawText) return []
  return props.rawText.split('\n').map((line, lineIdx) => {
    const tokens = []
    const regex = /(\S+|\s+)/g
    let match
    while ((match = regex.exec(line)) !== null) {
      const text = match[0]
      const isWord = /\S/.test(text)
      tokens.push({
        text,
        isWord,
        startIndex: match.index,
      })
    }
    return {
      lineIdx,
      line,
      tokens,
    }
  })
})

// Popover state
const activeWord = ref(null)
const copiedAction = ref(null) // 'word' | 'sentence' | 'line' | null
const arrowOffset = ref(110)
let hideTimer = null

function clearHideTimer() {
  if (hideTimer) {
    clearTimeout(hideTimer)
    hideTimer = null
  }
}

function scheduleHide() {
  clearHideTimer()
  hideTimer = setTimeout(() => {
    activeWord.value = null
    copiedAction.value = null
  }, 220)
}

function onWordMouseEnter(event, token, line, lineIdx, tokenIdx) {
  clearHideTimer()
  const el = event.currentTarget
  const rect = el.getBoundingClientRect()

  // Strip wrapping punctuation for clean word copy (e.g. "Total:" -> "Total", "125.00," -> "125.00")
  const cleanWord = token.text.replace(/^[.,;:!?"'()[\]{}]+|[.,;:!?"'()[\]{}]+$/g, '') || token.text
  const sentence = getSentenceForToken(line, token.startIndex)
  const lineText = line.trim()

  const popoverWidth = 240
  const popoverHeight = 135

  // Determine placement above or below
  const placement = rect.top < popoverHeight + 15 ? 'bottom' : 'top'
  const wordCenterX = rect.left + rect.width / 2

  // Clamp X so popover stays on screen
  const padding = 12
  const minX = popoverWidth / 2 + padding
  const maxX = window.innerWidth - (popoverWidth / 2) - padding
  const clampedX = Math.max(minX, Math.min(maxX, wordCenterX))

  // Calculate arrow offset relative to popover left
  const arrowX = wordCenterX - (clampedX - popoverWidth / 2)
  arrowOffset.value = Math.max(16, Math.min(popoverWidth - 16, arrowX))

  activeWord.value = {
    rawToken: token.text,
    cleanWord,
    sentence,
    line: lineText,
    isSameSentenceAndLine: sentence === lineText,
    lineIdx,
    tokenIdx,
    x: clampedX,
    y: placement === 'top' ? rect.top - 8 : rect.bottom + 8,
    placement,
  }
}

function onWordMouseLeave() {
  scheduleHide()
}

function onPopoverMouseEnter() {
  clearHideTimer()
}

function onPopoverMouseLeave() {
  scheduleHide()
}

// Quick click on word to copy word
function onWordClick(event, token) {
  const selection = window.getSelection()?.toString()
  if (selection && selection.length > 0) return

  const cleanWord = token.text.replace(/^[.,;:!?"'()[\]{}]+|[.,;:!?"'()[\]{}]+$/g, '') || token.text
  copyAction('word', cleanWord, 'Word Copied')
}

// Perform copy action from popover
async function copyAction(type, text, title) {
  if (!text) return
  copiedAction.value = type
  await copyToClipboard(text, title, `"${truncate(text, 32)}"`)

  setTimeout(() => {
    copiedAction.value = null
    activeWord.value = null
  }, 700)
}

// Handle global dismiss
function handleGlobalDismiss(e) {
  if (e?.type === 'keydown' && e.key !== 'Escape') return
  activeWord.value = null
}

onMounted(() => {
  window.addEventListener('scroll', handleGlobalDismiss, true)
  window.addEventListener('keydown', handleGlobalDismiss)
})

onUnmounted(() => {
  clearHideTimer()
  if (toastTimer) clearTimeout(toastTimer)
  window.removeEventListener('scroll', handleGlobalDismiss, true)
  window.removeEventListener('keydown', handleGlobalDismiss)
})

const popoverStyle = computed(() => {
  if (!activeWord.value) return {}
  const { x, y, placement } = activeWord.value
  return {
    left: `${x}px`,
    top: `${y}px`,
    transform: placement === 'top' ? 'translate(-50%, -100%)' : 'translate(-50%, 0)',
  }
})
</script>

<template>
  <div class="inspector-card">
    <div class="inspector-header">
      <div class="tab-buttons">
        <button
          type="button"
          class="tab-btn"
          :class="{ 'tab-btn-active': activeTab === 'text' }"
          @click="activeTab = 'text'"
        >
          📝 Raw OCR Text ({{ (rawText || '').split('\n').filter(Boolean).length }} lines)
        </button>
        <button
          type="button"
          class="tab-btn"
          :class="{ 'tab-btn-active': activeTab === 'json' }"
          @click="activeTab = 'json'"
        >
          ⚙️ Raw JSON Response
        </button>
      </div>

      <div class="header-actions">
        <span v-if="activeTab === 'text' && rawText" class="header-hint">
          💡 Hover word for copy options
        </span>
        <button
          type="button"
          class="copy-btn"
          @click="copyToClipboard(
            activeTab === 'text' ? rawText : JSON.stringify(rawResponse, null, 2),
            activeTab === 'text' ? 'Raw Text Copied' : 'JSON Copied',
            activeTab === 'text' ? 'All lines copied to clipboard' : 'JSON payload copied to clipboard'
          )"
        >
          Copy {{ activeTab === 'text' ? 'Text' : 'JSON' }}
        </button>
      </div>
    </div>

    <div class="inspector-content">
      <div v-if="activeTab === 'text'" class="text-view">
        <div v-if="!rawText" class="empty-state">No OCR text extracted yet.</div>
        <div v-else class="code-lines">
          <div
            v-for="item in parsedLines"
            :key="item.lineIdx"
            class="code-line"
          >
            <span class="line-no">{{ item.lineIdx + 1 }}</span>
            <span class="line-content">
              <template v-for="(token, tIdx) in item.tokens" :key="tIdx">
                <span
                  v-if="token.isWord"
                  class="ocr-word"
                  :class="{
                    'is-hovered': activeWord?.lineIdx === item.lineIdx && activeWord?.tokenIdx === tIdx
                  }"
                  @mouseenter="onWordMouseEnter($event, token, item.line, item.lineIdx, tIdx)"
                  @mouseleave="onWordMouseLeave"
                  @click="onWordClick($event, token)"
                >{{ token.text }}</span>
                <span v-else class="ocr-space">{{ token.text }}</span>
              </template>
              <span v-if="item.tokens.length === 0">&nbsp;</span>
            </span>

            <button
              v-if="item.line.trim()"
              type="button"
              class="line-copy-btn"
              title="Copy entire line"
              @click="copyAction('line', item.line.trim(), 'Line Copied')"
            >
              📋
            </button>
          </div>
        </div>
      </div>

      <div v-else class="json-view">
        <pre class="json-pre"><code>{{ JSON.stringify(rawResponse, null, 2) }}</code></pre>
      </div>
    </div>

    <!-- Floating Word Options Popover -->
    <Teleport to="body">
      <div
        v-if="activeWord"
        class="ocr-word-popover"
        :class="`placement-${activeWord.placement}`"
        :style="popoverStyle"
        @mouseenter="onPopoverMouseEnter"
        @mouseleave="onPopoverMouseLeave"
      >
        <div class="popover-header">
          <span class="popover-badge">WORD</span>
          <span class="popover-word-preview" :title="activeWord.cleanWord">
            "{{ truncate(activeWord.cleanWord, 18) }}"
          </span>
        </div>

        <div class="popover-actions">
          <!-- Option 1: Copy Word -->
          <button
            type="button"
            class="popover-item"
            :class="{ 'is-success': copiedAction === 'word' }"
            @click="copyAction('word', activeWord.cleanWord, 'Word Copied')"
          >
            <span class="item-icon">{{ copiedAction === 'word' ? '✓' : '📋' }}</span>
            <div class="item-text">
              <span class="item-title">{{ copiedAction === 'word' ? 'Copied Word!' : 'Copy Word' }}</span>
              <span class="item-snippet">"{{ truncate(activeWord.cleanWord, 20) }}"</span>
            </div>
          </button>

          <!-- Option 2: Copy Sentence -->
          <button
            type="button"
            class="popover-item"
            :class="{ 'is-success': copiedAction === 'sentence' }"
            @click="copyAction('sentence', activeWord.sentence, 'Sentence Copied')"
          >
            <span class="item-icon">{{ copiedAction === 'sentence' ? '✓' : '💬' }}</span>
            <div class="item-text">
              <span class="item-title">{{ copiedAction === 'sentence' ? 'Copied Sentence!' : 'Copy Sentence' }}</span>
              <span class="item-snippet">"{{ truncate(activeWord.sentence, 22) }}"</span>
            </div>
          </button>

          <!-- Option 3: Copy Line (if different from sentence) -->
          <button
            v-if="!activeWord.isSameSentenceAndLine"
            type="button"
            class="popover-item"
            :class="{ 'is-success': copiedAction === 'line' }"
            @click="copyAction('line', activeWord.line, 'Line Copied')"
          >
            <span class="item-icon">{{ copiedAction === 'line' ? '✓' : '📄' }}</span>
            <div class="item-text">
              <span class="item-title">{{ copiedAction === 'line' ? 'Copied Line!' : 'Copy Line' }}</span>
              <span class="item-snippet">"{{ truncate(activeWord.line, 22) }}"</span>
            </div>
          </button>
        </div>

        <!-- Arrow pointer -->
        <div class="popover-arrow" :style="{ left: `${arrowOffset}px` }"></div>
      </div>
    </Teleport>

    <!-- Sleek Toast Notification -->
    <Teleport to="body">
      <Transition name="ocr-toast-anim">
        <div v-if="toastMessage" class="ocr-toast-banner">
          <div class="toast-check-icon">✓</div>
          <div class="toast-content">
            <div class="toast-heading">{{ toastTitle }}</div>
            <div class="toast-subtext">{{ toastMessage }}</div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.inspector-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  overflow: hidden;
}
.inspector-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  background: #0d1521;
  border-bottom: 1px solid var(--border);
}
.tab-buttons {
  display: flex;
  gap: 8px;
}
.tab-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 600;
  padding: 6px 12px;
  border-radius: 6px;
  transition: all 0.15s;
}
.tab-btn:hover {
  color: var(--text-main);
}
.tab-btn-active {
  background: #192738;
  color: #60a5fa;
}
.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}
.header-hint {
  font-size: 11px;
  color: #64748b;
  font-weight: 500;
  background: rgba(100, 116, 139, 0.12);
  padding: 3px 8px;
  border-radius: 4px;
  border: 1px dashed rgba(100, 116, 139, 0.25);
}
.copy-btn {
  background: transparent;
  border: 1px solid var(--border);
  color: var(--text-dim);
  font-size: 11px;
  padding: 4px 10px;
  border-radius: 4px;
  transition: all 0.15s;
}
.copy-btn:hover {
  color: var(--text-main);
  border-color: var(--primary);
}
.inspector-content {
  max-height: 400px;
  overflow: auto;
  background: #070b12;
  font-family: var(--font-mono);
  font-size: 12px;
}
.empty-state {
  padding: 30px;
  text-align: center;
  color: var(--text-dim);
}
.code-lines {
  display: flex;
  flex-direction: column;
}
.code-line {
  display: flex;
  align-items: baseline;
  padding: 2px 12px;
  line-height: 1.6;
  position: relative;
}
.code-line:hover {
  background: rgba(255, 255, 255, 0.03);
}
.line-no {
  width: 38px;
  color: var(--text-dim);
  user-select: none;
  text-align: right;
  margin-right: 14px;
  flex-shrink: 0;
}
.line-content {
  color: #94a3b8;
  white-space: pre-wrap;
  word-break: break-all;
  flex: 1;
}

/* Individual OCR Word Styles */
.ocr-word {
  display: inline-block;
  cursor: pointer;
  border-radius: 3px;
  padding: 0 2px;
  margin: 0 -1px;
  transition: all 0.12s ease;
  position: relative;
}
.ocr-word:hover,
.ocr-word.is-hovered {
  background: rgba(59, 130, 246, 0.22);
  color: #93c5fd;
  text-decoration: underline dotted #60a5fa;
  box-shadow: 0 0 0 1px rgba(96, 165, 250, 0.4);
}
.ocr-space {
  white-space: pre;
}

/* Line copy button */
.line-copy-btn {
  background: transparent;
  border: 1px solid transparent;
  color: #64748b;
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 4px;
  margin-left: 8px;
  opacity: 0;
  transition: all 0.15s ease;
  cursor: pointer;
  user-select: none;
}
.code-line:hover .line-copy-btn {
  opacity: 0.7;
}
.line-copy-btn:hover {
  opacity: 1 !important;
  background: #1e293b;
  border-color: #3b82f6;
  color: #93c5fd;
}

.json-view {
  padding: 14px 18px;
}
.json-pre {
  color: #38bdf8;
  white-space: pre-wrap;
  word-break: break-all;
  line-height: 1.5;
}
</style>

<!-- Global Styles for Teleported Elements -->
<style>
/* Floating Word Popover */
.ocr-word-popover {
  position: fixed;
  z-index: 10000;
  width: 240px;
  background: #0f172a;
  border: 1px solid #2d3f56;
  border-radius: 10px;
  box-shadow: 0 14px 35px -6px rgba(0, 0, 0, 0.75), 0 0 0 1px rgba(255, 255, 255, 0.05);
  padding: 6px;
  font-family: var(--font-sans, system-ui, -apple-system, sans-serif);
  user-select: none;
  backdrop-filter: blur(12px);
  animation: ocrPopoverIn 0.14s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes ocrPopoverIn {
  from {
    opacity: 0;
    transform: translate(-50%, -92%) scale(0.96);
  }
  to {
    opacity: 1;
  }
}

.ocr-word-popover.placement-bottom {
  animation-name: ocrPopoverInBottom;
}

@keyframes ocrPopoverInBottom {
  from {
    opacity: 0;
    transform: translate(-50%, 8px) scale(0.96);
  }
  to {
    opacity: 1;
  }
}

.ocr-word-popover .popover-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 6px 6px;
  border-bottom: 1px solid #1e293b;
  margin-bottom: 4px;
}
.ocr-word-popover .popover-badge {
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.05em;
  background: rgba(59, 130, 246, 0.2);
  color: #60a5fa;
  padding: 1px 5px;
  border-radius: 3px;
}
.ocr-word-popover .popover-word-preview {
  font-size: 11px;
  color: #94a3b8;
  font-family: var(--font-mono, monospace);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ocr-word-popover .popover-actions {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.ocr-word-popover .popover-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 8px;
  border-radius: 6px;
  border: 1px solid transparent;
  background: transparent;
  color: #e2e8f0;
  text-align: left;
  cursor: pointer;
  transition: all 0.12s ease;
}
.ocr-word-popover .popover-item:hover {
  background: #1e293b;
  border-color: #334155;
  color: #60a5fa;
}
.ocr-word-popover .popover-item.is-success {
  background: rgba(16, 185, 129, 0.15) !important;
  border-color: rgba(16, 185, 129, 0.4) !important;
  color: #34d399 !important;
}

.ocr-word-popover .item-icon {
  font-size: 13px;
  flex-shrink: 0;
  width: 16px;
  text-align: center;
}
.ocr-word-popover .item-text {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  flex: 1;
}
.ocr-word-popover .item-title {
  font-size: 11px;
  font-weight: 600;
  line-height: 1.2;
}
.ocr-word-popover .item-snippet {
  font-size: 10px;
  color: #64748b;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.3;
}
.ocr-word-popover .popover-item:hover .item-snippet {
  color: #94a3b8;
}

/* Arrow Pointer */
.ocr-word-popover .popover-arrow {
  position: absolute;
  width: 0;
  height: 0;
  pointer-events: none;
}
.ocr-word-popover.placement-top .popover-arrow {
  bottom: -6px;
  transform: translateX(-50%);
  border-left: 6px solid transparent;
  border-right: 6px solid transparent;
  border-top: 6px solid #2d3f56;
}
.ocr-word-popover.placement-bottom .popover-arrow {
  top: -6px;
  transform: translateX(-50%);
  border-left: 6px solid transparent;
  border-right: 6px solid transparent;
  border-bottom: 6px solid #2d3f56;
}

/* Toast Banner */
.ocr-toast-banner {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 10001;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  background: #0d1521;
  border: 1px solid #10b981;
  border-radius: 8px;
  box-shadow: 0 10px 30px -5px rgba(0, 0, 0, 0.7), 0 0 16px rgba(16, 185, 129, 0.25);
  font-family: var(--font-sans, system-ui, -apple-system, sans-serif);
  color: #f1f5f9;
}
.ocr-toast-banner .toast-check-icon {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: rgba(16, 185, 129, 0.2);
  color: #10b981;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
  font-size: 13px;
  flex-shrink: 0;
}
.ocr-toast-banner .toast-content {
  display: flex;
  flex-direction: column;
}
.ocr-toast-banner .toast-heading {
  font-size: 12px;
  font-weight: 600;
  color: #f1f5f9;
}
.ocr-toast-banner .toast-subtext {
  font-size: 11px;
  color: #94a3b8;
  max-width: 280px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Toast Transitions */
.ocr-toast-anim-enter-active,
.ocr-toast-anim-leave-active {
  transition: all 0.22s cubic-bezier(0.16, 1, 0.3, 1);
}
.ocr-toast-anim-enter-from {
  opacity: 0;
  transform: translateY(12px) scale(0.95);
}
.ocr-toast-anim-leave-to {
  opacity: 0;
  transform: translateY(12px) scale(0.95);
}
</style>
