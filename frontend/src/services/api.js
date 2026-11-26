// When running in Capacitor APK on Android, point to the Rust backend server on LAN
const isNativeApp = typeof window !== 'undefined' && (
  window.Capacitor?.isNativePlatform?.() ||
  window.location?.protocol === 'capacitor:' ||
  (window.location?.hostname === 'localhost' && window.location?.port !== '5173')
)
const API_BASE = import.meta.env.VITE_API_BASE || (isNativeApp ? 'http://192.168.0.221:8081' : '')

export async function checkHealth() {
  try {
    const res = await fetch(`${API_BASE}/api/health`)
    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    return await res.json()
  } catch (err) {
    return { status: 'offline', error: err.message }
  }
}

async function optimizeImageIfLarge(file) {
  if (!file || !file.type.startsWith('image/')) return file
  // Only optimize if larger than 2MB
  if (file.size <= 2 * 1024 * 1024) return file

  return new Promise((resolve) => {
    const img = new Image()
    const url = URL.createObjectURL(file)

    img.onload = () => {
      URL.revokeObjectURL(url)
      const maxDim = 2560
      let { width, height } = img

      if (width > maxDim || height > maxDim) {
        if (width > height) {
          height = Math.round((height * maxDim) / width)
          width = maxDim
        } else {
          width = Math.round((width * maxDim) / height)
          height = maxDim
        }
      }

      const canvas = document.createElement('canvas')
      canvas.width = width
      canvas.height = height
      const ctx = canvas.getContext('2d')
      ctx.drawImage(img, 0, 0, width, height)

      canvas.toBlob(
        (blob) => {
          if (blob && blob.size < file.size) {
            const fileName = (file.name || 'document').replace(/\.[^.]+$/, '') + '.jpg'
            const optimized = new File([blob], fileName, {
              type: 'image/jpeg',
            })
            resolve(optimized)
          } else {
            resolve(file)
          }
        },
        'image/jpeg',
        0.92
      )
    }

    img.onerror = () => {
      URL.revokeObjectURL(url)
      resolve(file)
    }

    img.src = url
  })
}

export async function scanDocument(file, onProgress = () => {}) {
  if (!file) throw new Error('No file selected.')

  onProgress('Preparing document...')
  const uploadFile = await optimizeImageIfLarge(file)

  const formData = new FormData()
  const safeFileName = uploadFile.name || 'document.jpg'
  formData.append('file', uploadFile, safeFileName)

  onProgress('Uploading document to Rust scanner...')
  const res = await fetch(`${API_BASE}/api/documents/scan`, {
    method: 'POST',
    body: formData,
  })

  if (!res.ok) {
    let errorDetail = `Scan failed with status ${res.status}`
    try {
      const errJson = await res.json()
      if (errJson.error) errorDetail = errJson.error
    } catch {
      // ignore
    }
    throw new Error(errorDetail)
  }

  return await res.json()
}
