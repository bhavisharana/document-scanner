# Rust Document Scanner & OCR Service

A high-performance backend microservice built in **Rust** (using **Axum**, **Tokio**, and **Tesseract OCR**) for extracting text and structured business data (Delivery Challans, Tax Invoices, Receipts, etc.) from PDFs and images.

> **📦 New to this project or setting up from a ZIP file?**  
> Check out the complete step-by-step setup guide: **[SETUP_GUIDE.md](SETUP_GUIDE.md)**.

---

## Features

- ⚡ **Hybrid Extraction Pipeline**:
  - **Digital PDFs**: Direct text extraction in ~10–25ms using native Rust text stream parsers.
  - **Scanned PDFs**: Automatically rasterizes pages to high-DPI images via `pdftoppm` and runs multi-page OCR.
  - **Images**: Enhanced preprocessing (automatic upscaling, grayscale normalization, contrast boost) before OCR.
- 🎯 **Accurate Field Heuristics**:
  - Document Type (`Delivery Challan`, `Tax Invoice`, `Receipt`, `Purchase Order`)
  - Voucher / Challan / Invoice Number
  - Date & Due Date
  - Vehicle Number (standard Indian formats: `GJ 01 AB 1234`, `MH-12-CD-5678`, etc.)
  - Party / Customer / Vendor Name
  - GSTIN Identification
  - Subtotal, Tax Amount, and Grand Total
  - Tabular Line Items (Sr No, Item Description, Quantity, Rate, Amount)
- 📊 **Confidence Ratings**: Computes mean OCR word confidence scores from Tesseract TSV output.
- 🌐 **Web Ready**: Built-in permissive CORS support for frontends (Vue, React, Angular, Next.js, Laravel).
- 🐳 **Containerized**: Production-ready multi-stage `Dockerfile`.

---

## Directory Structure

```text
document_scanner/
├── Cargo.toml                  # Cargo dependencies & project metadata
├── Dockerfile                  # Multi-stage production container
├── .env.example                # Example environment variables
├── README.md                   # Documentation & API reference
└── src/
    ├── main.rs                 # Application entrypoint & HTTP server
    ├── config.rs               # Environment variable configuration
    ├── error.rs                # Error handling & JSON responses
    ├── models/
    │   ├── mod.rs
    │   └── document.rs         # Data structures (ExtractedFields, LineItem, ScanResponse)
    ├── routes/
    │   ├── mod.rs
    │   └── scan.rs             # /api/documents/scan and /api/health handlers
    └── services/
        ├── mod.rs
        ├── ocr.rs              # Tesseract OCR engine & TSV confidence parser
        ├── parser.rs           # Regex & heuristic structured data extractor
        ├── pdf.rs              # Hybrid digital & scanned PDF processing
        └── preprocessor.rs     # Image upscaling, grayscale & contrast optimization
```

---

## Prerequisites

On your host machine (or inside Docker):
- **Rust** 1.80+ (`rustc` and `cargo`)
- **Tesseract OCR**:
  ```bash
  sudo apt-get update && sudo apt-get install -y tesseract-ocr tesseract-ocr-eng poppler-utils
  ```

---

## Running Locally

### Option 1: All-In-One (Rust Backend + Vue 3 Frontend)
The Rust backend automatically serves the Vue 3 frontend if `frontend/dist` is built:

1. **Build Frontend (one-time or after edits)**:
   ```bash
   cd frontend
   npm install
   npm run build
   cd ..
   ```

2. **Run Rust Server**:
   ```bash
   export PATH="$HOME/.cargo/bin:$PATH"
   cargo run
   ```
   Open **http://localhost:8081** in your browser. You get the full UI with live document preview, OCR scanning, and the Delivery Challan Alteration form.

---

### Option 2: Frontend Vite Dev Mode (with Hot Reload)
1. **Run Rust Backend in one terminal**:
   ```bash
   cargo run
   ```
2. **Run Vue 3 Dev Server in another terminal**:
   ```bash
   cd frontend
   npm run dev
   ```
   Open **http://localhost:5173** in your browser. It automatically proxies `/api` calls to the Rust backend on port `8081`.

---

## API Documentation

### 1. Health Check
- **URL**: `GET /api/health`
- **Response**:
  ```json
  {
    "status": "healthy",
    "service": "rust-document-scanner",
    "version": "0.1.0",
    "tesseract_path": "/usr/bin/tesseract",
    "max_file_size_mb": 25
  }
  ```

---

### 2. Scan Document
- **URL**: `POST /api/documents/scan`
- **Content-Type**: `multipart/form-data`
- **Form Field**: `file` (PDF, PNG, JPG, JPEG, WEBP, or TIFF)

#### Example with cURL:
```bash
curl -X POST http://localhost:8081/api/documents/scan \
  -F "file=@/path/to/invoice_or_challan.pdf"
```

#### Example Response:
```json
{
  "success": true,
  "fields": {
    "document_type": "Delivery Challan",
    "voucher_no": "DC/2026/001",
    "date": "15/03/2026",
    "due_date": null,
    "party_name": "ABC Steel Industries Pvt Ltd",
    "party_address": null,
    "gst_no": "24AAACA1234A1Z5",
    "vehicle_no": "GJ 01 AB 1234",
    "subtotal": 40000.0,
    "tax_amount": 5200.50,
    "total_amount": "45200.50",
    "items": [
      {
        "sr_no": 1,
        "description": "Steel Rods",
        "hsn_sac": null,
        "quantity": 25.0,
        "unit": "pcs",
        "rate": 120.0,
        "amount": 3000.0
      }
    ],
    "notes": null
  },
  "raw_text": "DELIVERY CHALLAN\nChallan No: DC/2026/001\nDate: 15/03/2026...",
  "confidence": 94.0,
  "processing_time_ms": 142,
  "page_count": 1,
  "source_type": "digital_pdf"
}
```

---

## Docker Deployment

To build and run in Docker:

```bash
# Build Docker image
docker build -t rust-document-scanner .

# Run container
docker run -d -p 8081:8081 --name doc-scanner rust-document-scanner
```
