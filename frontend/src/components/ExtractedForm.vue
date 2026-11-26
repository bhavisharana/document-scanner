<script setup>
import { ref, computed } from 'vue'

const props = defineProps({
  modelValue: { type: Object, required: true },
  confidence: { type: Number, default: 0 },
  processingTimeMs: { type: Number, default: 0 },
  sourceType: { type: String, default: 'image' },
  busy: { type: Boolean, default: false },
})

const emit = defineEmits(['update:modelValue', 'save'])

const confidenceColor = computed(() => {
  if (props.confidence >= 80) return 'var(--success)'
  if (props.confidence >= 50) return 'var(--warning)'
  return 'var(--danger)'
})

const confidenceBadgeClass = computed(() => {
  if (props.confidence >= 80) return 'badge-success'
  if (props.confidence >= 50) return 'badge-warning'
  return 'badge-danger'
})

const gstCopied = ref(false)

const GST_STATE_MAP = {
  '01': 'Jammu & Kashmir', '02': 'Himachal Pradesh', '03': 'Punjab', '04': 'Chandigarh',
  '05': 'Uttarakhand', '06': 'Haryana', '07': 'Delhi', '08': 'Rajasthan',
  '09': 'Uttar Pradesh', '10': 'Bihar', '11': 'Sikkim', '12': 'Arunachal Pradesh',
  '13': 'Nagaland', '14': 'Manipur', '15': 'Mizoram', '16': 'Tripura',
  '17': 'Meghalaya', '18': 'Assam', '19': 'West Bengal', '20': 'Jharkhand',
  '21': 'Odisha', '22': 'Chhattisgarh', '23': 'Madhya Pradesh', '24': 'Gujarat',
  '25': 'Daman & Diu', '26': 'Dadra & Nagar Haveli', '27': 'Maharashtra', '28': 'Andhra Pradesh',
  '29': 'Karnataka', '30': 'Goa', '31': 'Lakshadweep', '32': 'Kerala',
  '33': 'Tamil Nadu', '34': 'Puducherry', '35': 'Andaman & Nicobar', '36': 'Telangana',
  '37': 'Andhra Pradesh', '38': 'Ladakh', '97': 'Other Territory', '99': 'Centre Jurisdiction'
}

const gstInfo = computed(() => {
  const raw = (props.modelValue.gst_no || '').trim().toUpperCase()
  if (!raw) {
    return { exists: false, isValid: false, isComplete: false, raw: '', stateCode: '', state: '', pan: '', charCount: 0 }
  }

  const charCount = raw.length
  const stateCode = raw.slice(0, 2)
  const state = GST_STATE_MAP[stateCode] || (stateCode.length === 2 && /^\d{2}$/.test(stateCode) ? `State Code ${stateCode}` : '')
  const pan = raw.length >= 12 ? raw.slice(2, 12) : ''
  const isComplete = charCount === 15

  // Standard Indian GSTIN: 2 state digits + 5 PAN letters + 4 PAN digits + 1 PAN letter + 1 entity + Z + 1 checksum
  const isValid = /^[0-9]{2}[A-Z]{5}[0-9]{4}[A-Z]{1}[1-9A-Z]{1}Z[0-9A-Z]{1}$/.test(raw)

  return {
    exists: true,
    raw,
    charCount,
    stateCode,
    state,
    pan,
    isComplete,
    isValid,
  }
})

function onGstInput(event) {
  const raw = event.target.value || ''
  const formatted = raw.toUpperCase().replace(/[^0-9A-Z]/g, '').slice(0, 15)
  updateField('gst_no', formatted)
}

function copyGst() {
  const val = props.modelValue.gst_no || ''
  if (!val) return
  navigator.clipboard.writeText(val)
  gstCopied.value = true
  setTimeout(() => {
    gstCopied.value = false
  }, 1500)
}

function onSave() {
  if (gstInfo.value.exists && gstInfo.value.charCount !== 15) {
    alert(`Party GST Number must be exactly 15 characters`)
    return
  }
  emit('save')
}

function updateField(key, value) {
  const updated = { ...props.modelValue, [key]: value }
  if (key === 'voucher_no') updated.documentNumber = value
  if (key === 'date') updated.documentDate = value
  if (key === 'party') updated.party_name = value
  if (key === 'party_name') updated.party = value
  if (key === 'party_address') updated.address = value
  if (key === 'taxable') updated.subtotal = parseFloat(value) || 0
  if (key === 'gst_no') {
    updated.gst_no = value
  }
  emit('update:modelValue', updated)
}

function updateItem(index, key, value) {
  const currentItems = [...(props.modelValue.items || [])]
  if (!currentItems[index]) return
  currentItems[index] = { ...currentItems[index], [key]: value }

  // Sync aliases
  if (key === 'quality') {
    currentItems[index].description = value
    currentItems[index].name = value
  }
  if (key === 'design') {
    currentItems[index].hsn_sac = value
  }
  if (key === 'qty') {
    currentItems[index].quantity = parseFloat(value) || 0
  }
  if (key === 'rate') {
    currentItems[index].price = value
  }

  // Recalculate line amount if qty or rate changes
  if (key === 'qty' || key === 'rate') {
    const q = parseFloat(currentItems[index].qty || currentItems[index].quantity)
    const r = parseFloat(currentItems[index].rate || currentItems[index].price)
    if (!isNaN(q) && !isNaN(r) && q > 0 && r >= 0) {
      currentItems[index].amount = parseFloat((q * r).toFixed(2))
    }
  }

  // Update total rolls/qty and taxable automatically
  recalculateTotals(currentItems)
}

function addItem() {
  const currentItems = [...(props.modelValue.items || [])]
  currentItems.push({
    sr_no: currentItems.length + 1,
    quality: '',
    description: '',
    name: '',
    design: '',
    hsn_sac: '',
    shade: '',
    qty: '1.00',
    quantity: 1.0,
    unit: 'PCS',
    rate: '',
    price: '',
    disc: '',
    gst: '18%',
    tax: '18%',
    amount: 0,
  })
  recalculateTotals(currentItems)
}

function removeItem(index) {
  const currentItems = [...(props.modelValue.items || [])]
  currentItems.splice(index, 1)
  recalculateTotals(currentItems)
}

function recalculateTotals(items) {
  let sumQty = 0
  let sumTaxable = 0
  let unit = 'PCS'

  items.forEach((it) => {
    const q = parseFloat(it.qty || it.quantity) || 0
    const a = parseFloat(it.amount) || 0
    sumQty += q
    sumTaxable += a
    if (it.unit) unit = it.unit
  })

  const updated = {
    ...props.modelValue,
    items,
    rollsQty: `${items.length} / ${sumQty.toFixed(2)} ${unit}`,
    total_qty: sumQty,
  }

  // Only auto-update taxable if user hasn't explicitly fixed it
  if (!props.modelValue.taxable || props.modelValue.taxable === '0.00' || props.modelValue.taxable === '0') {
    updated.taxable = sumTaxable.toFixed(2)
    updated.subtotal = sumTaxable
  }

  emit('update:modelValue', updated)
}

function copyJson() {
  navigator.clipboard.writeText(JSON.stringify(props.modelValue, null, 2))
  alert('Scanned fields copied to clipboard as JSON!')
}
</script>

<template>
  <div class="form-container">
    <!-- Top Metadata Card -->
    <div class="header-card">
      <div class="header-main">
        <div class="title-row">
          <h2 class="form-title">{{ modelValue.document_type || 'Delivery Challan' }} Verification</h2>
          <span class="voucher-badge">Voucher: {{ modelValue.voucher_no || 'Auto' }}</span>
        </div>
        <p class="form-subtitle">Review, verify and edit the scanned fields below</p>
      </div>

      <div class="meta-badges">
        <div class="meta-badge" :class="confidenceBadgeClass">
          <span class="badge-dot"></span>
          <span>OCR Confidence: <b>{{ Math.round(confidence) }}%</b></span>
        </div>
        <div class="meta-badge badge-neutral">
          <span>⚡ Time: <b>{{ processingTimeMs }}ms</b></span>
        </div>
        <div class="meta-badge badge-neutral">
          <span>📄 Source: <b>{{ sourceType }}</b></span>
        </div>
      </div>
    </div>

    <!-- Form Section 1: Header Fields -->
    <div class="card-section">
      <div class="section-title">
        <span>Document & Party Information</span>
      </div>

      <div class="party-info-grid">
        <!-- Row 1: First 3 fields -->
        <div class="grid-fields-3">
          <div class="field-group">
            <label class="field-label">Voucher No</label>
            <input
              type="text"
              class="input-field"
              :value="modelValue.voucher_no || ''"
              placeholder="e.g. 32 or DC-001"
              @input="updateField('voucher_no', $event.target.value)"
            />
          </div>

          <div class="field-group">
            <label class="field-label">Date (DD/MM/YYYY)</label>
            <input
              type="text"
              class="input-field"
              :value="modelValue.date || ''"
              placeholder="DD/MM/YYYY"
              @input="updateField('date', $event.target.value)"
            />
          </div>

          <div class="field-group">
            <label class="field-label">Party / Customer Name</label>
            <input
              type="text"
              class="input-field"
              :value="modelValue.party || modelValue.party_name || ''"
              placeholder="e.g. Mahindra Mechanical Works"
              @input="updateField('party', $event.target.value)"
            />
          </div>
        </div>

        <!-- Row 2: Other 2 fields -->
        <div class="grid-fields-2">
          <div class="field-group">
            <label class="field-label">Broker</label>
            <input
              type="text"
              class="input-field"
              :value="modelValue.broker || ''"
              placeholder="Broker name (optional)"
              @input="updateField('broker', $event.target.value)"
            />
          </div>
          
          <div class="field-group gst-field-group">
            <div class="field-label-row">
              <label class="field-label">Party GST No. / GSTIN</label>
            </div>

            <div class="gst-input-wrapper">
              <input
                type="text"
                class="input-field gst-input"
                :class="{
                  'input-valid': gstInfo.isValid,
                  'input-warning': gstInfo.isComplete && !gstInfo.isValid,
                  'input-error': gstInfo.exists && !gstInfo.isComplete
                }"
                :value="modelValue.gst_no || ''"
                placeholder="e.g. 09AKUPA2408E1ZC"
                maxlength="15"
                @input="onGstInput"
              />
            </div>

            <!-- Live Validation & State / PAN Breakdown -->
            <div v-if="gstInfo.exists" class="gst-status-hint">
              <span v-if="gstInfo.isValid" class="status-valid">
                ✓ Valid 15-character GSTIN
              </span>
              <span v-else-if="gstInfo.isComplete" class="status-invalid">
                ⚠️ 15 characters, but invalid GST structure
              </span>
              <span v-else class="status-error">
                ❌ GST number must be exactly 15 characters
              </span>
            </div>
          </div>
        </div>

        <!-- Row 3: Address -->
        <div class="field-group full-width">
          <label class="field-label">Party / Customer Address</label>
          <input
            type="text"
            class="input-field"
            :value="modelValue.party_address || modelValue.address || ''"
            placeholder="e.g. 04, KK Buildings, Ajmeri Gate, Jodhpur, Rajasthan"
            @input="updateField('party_address', $event.target.value)"
          />
        </div>
      </div>
    </div>

    <!-- Form Section 2: Items / Quality Grid -->
    <div class="card-section">
      <div class="table-header-row">
        <div class="section-title">
          <span>Items / Quality Grid</span>
          <span class="count-badge">{{ (modelValue.items || []).length }} Items</span>
        </div>
        <button type="button" class="btn-add-item" @click="addItem">
          <span>+</span> Add item
        </button>
      </div>

      <div v-if="!(modelValue.items && modelValue.items.length)" class="empty-table-notice">
        No line items detected yet. Click <b>+ Add item</b> to add products manually.
      </div>

      <div v-else class="table-responsive">
        <table class="items-table">
          <thead>
            <tr>
              <th style="width: 36px">#</th>
              <th>Quality / Description</th>
              <th style="width: 110px">Design / HSN</th>
              <th style="width: 90px">Shade</th>
              <th style="width: 80px">Qty</th>
              <th style="width: 95px">Rate</th>
              <th style="width: 80px">Disc %</th>
              <th style="width: 80px">GST %</th>
              <th style="width: 100px">Amount</th>
              <th style="width: 40px"></th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(item, idx) in modelValue.items" :key="idx">
              <td class="cell-idx">{{ idx + 1 }}</td>
              <td>
                <input
                  type="text"
                  class="table-input"
                  :value="item.quality || item.description || ''"
                  placeholder="e.g. Electric Drill Machine"
                  @input="updateItem(idx, 'quality', $event.target.value)"
                />
              </td>
              <td>
                <input
                  type="text"
                  class="table-input"
                  :value="item.design || item.hsn_sac || ''"
                  placeholder="HSN Code"
                  @input="updateItem(idx, 'design', $event.target.value)"
                />
              </td>
              <td>
                <input
                  type="text"
                  class="table-input"
                  :value="item.shade || ''"
                  placeholder="-"
                  @input="updateItem(idx, 'shade', $event.target.value)"
                />
              </td>
              <td>
                <input
                  type="text"
                  class="table-input table-input-num"
                  :value="item.qty || item.quantity || ''"
                  placeholder="1.00"
                  @input="updateItem(idx, 'qty', $event.target.value)"
                />
              </td>
              <td>
                <input
                  type="text"
                  class="table-input table-input-num"
                  :value="item.rate || item.price || ''"
                  placeholder="0.00"
                  @input="updateItem(idx, 'rate', $event.target.value)"
                />
              </td>
              <td>
                <input
                  type="text"
                  class="table-input"
                  :value="item.disc || ''"
                  placeholder="0%"
                  @input="updateItem(idx, 'disc', $event.target.value)"
                />
              </td>
              <td>
                <input
                  type="text"
                  class="table-input"
                  :value="item.gst || item.tax || ''"
                  placeholder="18%"
                  @input="updateItem(idx, 'gst', $event.target.value)"
                />
              </td>
              <td>
                <input
                  type="text"
                  class="table-input table-input-num bold-input"
                  :value="item.amount !== undefined ? item.amount : ''"
                  placeholder="0.00"
                  @input="updateItem(idx, 'amount', $event.target.value)"
                />
              </td>
              <td class="cell-action">
                <button
                  type="button"
                  class="btn-del-row"
                  title="Remove row"
                  @click="removeItem(idx)"
                >
                  ✕
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Form Section 3: Transport & Totals Grid -->
    <div class="bottom-split-grid">
      <!-- Remarks / Notes -->
      <div class="card-section flex-1">
        <div class="section-title">
          <span>Remarks & Narration</span>
        </div>

        <div class="field-group full-width" style="margin-top: 12px;">
          <label class="field-label">Narration / Notes</label>
          <textarea
            class="input-field"
            style="min-height: 140px; resize: vertical; line-height: 1.5;"
            :value="modelValue.notes || ''"
            placeholder="Optional remarks, payment terms, or notes"
            @input="updateField('notes', $event.target.value)"
          ></textarea>
        </div>
      </div>

      <!-- Financial Totals Summary (Matching Image 2) -->
      <div class="card-section totals-card">
        <div class="section-title">
          <span>Financial Totals</span>
        </div>

        <div class="totals-list">
          <div class="total-row">
            <span class="total-label">Rolls / Qty</span>
            <input
              type="text"
              class="total-input"
              :value="modelValue.rollsQty || ''"
              placeholder="0 / 0.00 PCS"
              @input="updateField('rollsQty', $event.target.value)"
            />
          </div>

          <div class="total-row">
            <span class="total-label">Taxable (₹)</span>
            <input
              type="text"
              class="total-input"
              :value="modelValue.taxable || ''"
              placeholder="0.00"
              @input="updateField('taxable', $event.target.value)"
            />
          </div>

          <div class="total-row">
            <span class="total-label">CGST (₹)</span>
            <input
              type="text"
              class="total-input"
              :value="modelValue.cgst || ''"
              placeholder="0.00"
              @input="updateField('cgst', $event.target.value)"
            />
          </div>

          <div class="total-row">
            <span class="total-label">SGST (₹)</span>
            <input
              type="text"
              class="total-input"
              :value="modelValue.sgst || ''"
              placeholder="0.00"
              @input="updateField('sgst', $event.target.value)"
            />
          </div>

          <div class="total-divider"></div>

          <div class="total-row total-grand-row">
            <span class="total-grand-label">Grand Total (₹)</span>
            <input
              type="text"
              class="total-grand-input"
              :value="modelValue.total_amount || modelValue.totalAmount || ''"
              placeholder="0.00"
              @input="updateField('total_amount', $event.target.value)"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Form Footer Actions -->
    <div class="form-footer">
      <button type="button" class="btn-secondary" @click="copyJson">
        📋 Copy JSON
      </button>
      <button type="button" class="btn-primary" :disabled="busy" @click="onSave">
        {{ busy ? 'Saving...' : '✓ Confirm & Save Data' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.form-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.header-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 16px 20px;
  flex-wrap: wrap;
  gap: 12px;
}
.title-row {
  display: flex;
  align-items: center;
  gap: 12px;
}
.form-title {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-main);
}
.voucher-badge {
  background: #1e2c3d;
  color: #60a5fa;
  font-size: 12px;
  font-family: var(--font-mono);
  padding: 2px 8px;
  border-radius: 4px;
  border: 1px solid rgba(96, 165, 250, 0.3);
}
.form-subtitle {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 2px;
}
.meta-badges {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.meta-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  padding: 4px 10px;
  border-radius: 20px;
  border: 1px solid transparent;
}
.badge-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
}
.badge-success {
  background: var(--success-bg);
  color: var(--success);
  border-color: rgba(16, 185, 129, 0.3);
}
.badge-warning {
  background: var(--warning-bg);
  color: var(--warning);
  border-color: rgba(245, 158, 11, 0.3);
}
.badge-danger {
  background: var(--danger-bg);
  color: var(--danger);
  border-color: rgba(239, 68, 68, 0.3);
}
.badge-neutral {
  background: #0d1521;
  color: var(--text-muted);
  border-color: var(--border);
}
.form-warning-alert {
  background: rgba(245, 158, 11, 0.12);
  border: 1px solid rgba(245, 158, 11, 0.45);
  color: #fde68a;
  padding: 14px 18px;
  border-radius: var(--radius-lg);
  display: flex;
  align-items: center;
  gap: 14px;
}
.form-warning-alert .warning-icon {
  font-size: 24px;
  line-height: 1;
}
.form-warning-alert .warning-title {
  font-weight: 700;
  font-size: 13px;
  color: #fbbf24;
  margin-bottom: 2px;
}
.form-warning-alert .warning-desc {
  font-size: 12px;
  color: #fef3c7;
  line-height: 1.4;
}
.card-section {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 18px 20px;
}
.section-title {
  font-size: 13px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-muted);
  margin-bottom: 14px;
  display: flex;
  align-items: center;
  gap: 8px;
}
.count-badge {
  background: var(--primary-glow);
  color: var(--primary);
  font-size: 10px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 10px;
}
.party-info-grid {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.grid-fields {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 14px;
}
.grid-fields-3 {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 14px;
}
.grid-fields-2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}
@media (max-width: 860px) {
  .grid-fields-3,
  .grid-fields-2 {
    grid-template-columns: 1fr;
  }
}
.field-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.full-width {
  grid-column: 1 / -1;
}
.field-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-muted);
}
.input-field {
  background: var(--bg-input);
  border: 1px solid var(--border);
  color: var(--text-main);
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 13px;
  transition: all 0.15s;
}
.input-field:focus {
  outline: none;
  border-color: var(--border-focus);
  box-shadow: 0 0 0 2px var(--primary-glow);
}
.table-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.btn-add-item {
  background: transparent;
  border: 1px solid var(--border);
  color: #60a5fa;
  padding: 5px 10px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  transition: all 0.15s;
}
.btn-add-item:hover {
  background: rgba(59, 130, 246, 0.1);
  border-color: var(--primary);
}
.empty-table-notice {
  text-align: center;
  padding: 20px;
  color: var(--text-dim);
  font-size: 13px;
  border: 1px dashed var(--border);
  border-radius: 6px;
}
.table-responsive {
  overflow-x: auto;
}
.items-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
}
.items-table th {
  background: #0d1521;
  color: var(--text-dim);
  font-weight: 600;
  text-align: left;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border);
  font-size: 11px;
}
.items-table td {
  padding: 6px 4px;
  border-bottom: 1px solid #142030;
}
.cell-idx {
  text-align: center;
  color: var(--text-dim);
  font-family: var(--font-mono);
  font-size: 11px;
}
.table-input {
  width: 100%;
  background: var(--bg-input);
  border: 1px solid transparent;
  color: var(--text-main);
  padding: 6px 8px;
  border-radius: 4px;
  font-size: 12px;
  transition: all 0.15s;
}
.table-input:hover {
  border-color: var(--border);
}
.table-input:focus {
  outline: none;
  border-color: var(--border-focus);
  background: #111e30;
}
.table-input-num {
  text-align: right;
  font-family: var(--font-mono);
}
.bold-input {
  font-weight: 600;
  color: #60a5fa;
}
.btn-del-row {
  background: transparent;
  border: none;
  color: var(--text-dim);
  font-size: 14px;
  width: 24px;
  height: 24px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}
.btn-del-row:hover {
  color: var(--danger);
  background: var(--danger-bg);
}
.bottom-split-grid {
  display: grid;
  grid-template-columns: 1fr 340px;
  gap: 16px;
}
@media (max-width: 900px) {
  .bottom-split-grid {
    grid-template-columns: 1fr;
  }
}
.totals-card {
  background: #0f1824;
}
.totals-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.total-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.total-label {
  font-size: 12px;
  color: var(--text-muted);
}
.total-input {
  width: 140px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  color: var(--text-main);
  padding: 6px 10px;
  border-radius: 6px;
  font-size: 12px;
  text-align: right;
  font-family: var(--font-mono);
}
.total-divider {
  height: 1px;
  background: var(--border);
  margin: 4px 0;
}
.total-grand-row {
  padding-top: 4px;
}
.total-grand-label {
  font-size: 13px;
  font-weight: 700;
  color: #ffffff;
}
.total-grand-input {
  width: 140px;
  background: rgba(59, 130, 246, 0.15);
  border: 1px solid rgba(59, 130, 246, 0.4);
  color: #60a5fa;
  padding: 8px 10px;
  border-radius: 6px;
  font-size: 15px;
  font-weight: 700;
  text-align: right;
  font-family: var(--font-mono);
}
.form-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
  padding: 10px 0;
}
.btn-secondary {
  background: transparent;
  border: 1px solid var(--border);
  color: var(--text-main);
  padding: 10px 18px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  transition: all 0.15s;
}
.btn-secondary:hover {
  background: var(--bg-card-hover);
  border-color: var(--text-muted);
}
.btn-primary {
  background: var(--primary);
  border: none;
  color: #ffffff;
  padding: 10px 22px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 700;
  box-shadow: 0 4px 14px var(--primary-glow);
  transition: all 0.15s;
}
.btn-primary:hover:not(:disabled) {
  background: var(--primary-hover);
  transform: translateY(-1px);
}

/* GST No. Custom Styles */
.field-label-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.gst-state-badge {
  font-size: 10px;
  font-weight: 700;
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.12);
  border: 1px solid rgba(56, 189, 248, 0.25);
  padding: 1px 6px;
  border-radius: 4px;
  letter-spacing: 0.2px;
}
.gst-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}
.gst-input {
  width: 100%;
  font-family: var(--font-mono, monospace);
  font-weight: 600;
  letter-spacing: 0.8px;
  text-transform: uppercase;
  padding-right: 56px;
}
.gst-input.input-valid {
  border-color: rgba(16, 185, 129, 0.6);
  background: rgba(16, 185, 129, 0.04);
}
.gst-input.input-warning {
  border-color: rgba(245, 158, 11, 0.6);
  background: rgba(245, 158, 11, 0.04);
}
.gst-actions-inline {
  position: absolute;
  right: 6px;
  display: flex;
  align-items: center;
  gap: 4px;
}
.btn-gst-action {
  background: #192738;
  border: 1px solid var(--border);
  color: var(--text-muted);
  width: 22px;
  height: 22px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  cursor: pointer;
  transition: all 0.15s ease;
  text-decoration: none;
}
.btn-gst-action:hover {
  background: #22374e;
  color: var(--text-main);
  border-color: var(--primary);
}
.btn-gst-portal {
  color: #60a5fa;
  font-weight: bold;
}
.gst-status-hint {
  font-size: 11px;
  line-height: 1.3;
  margin-top: 2px;
}
.status-valid {
  color: #10b981;
  font-weight: 500;
}
.status-invalid {
  color: #f59e0b;
  font-weight: 500;
}
.status-pending {
  color: var(--text-dim);
}
.status-error {
  color: #ef4444;
  font-weight: 500;
}
.gst-input.input-error {
  border-color: rgba(239, 68, 68, 0.6);
  background: rgba(239, 68, 68, 0.05);
}
</style>
