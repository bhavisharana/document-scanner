## 1. System Prerequisites

Before starting, ensure your system has:

- **Rust Toolchain**: 1.75+ (`rustc` and `cargo`)
- **Node.js**: **v20+ or v22+** (required for Vite 8 frontend build) and `npm`
- **Tesseract OCR**: For image and scanned document text recognition
- **Poppler Utilities** (`pdftotext` & `pdftoppm`): For layout and scanned PDF processing

---

## 2. Install Required Tools & Dependencies

### A. Ubuntu / Debian / Linux

Open your terminal and run:

```bash
# 1. Update package lists
sudo apt update

# 2. Install Tesseract OCR, English models, and Poppler utilities
sudo apt install -y tesseract-ocr tesseract-ocr-eng poppler-utils curl build-essential

# 3. Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# 4. Install Node.js (v20 or v22 via NVM or NodeSource)
# If you use NVM:
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
nvm install 20
nvm use 20
```

Verify your versions:

```bash
cargo --version       # Should be 1.75+
node -v               # Should be v20.x or v22.x
tesseract --version   # Should show Tesseract 4.x or 5.x
pdftotext -v          # Should show poppler/pdftotext version
```

---

### B. macOS

Using [Homebrew](https://brew.sh/):

```bash
# 1. Install OCR & PDF tools
brew install tesseract tesseract-lang poppler

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 3. Install Node.js
brew install node@20
brew link node@20
```

---

### C. Windows

#### Recommended: Use WSL2 (Windows Subsystem for Linux)

1. Open PowerShell as Administrator and run: `wsl --install`
2. Open Ubuntu in WSL2 and follow the **Ubuntu / Debian / Linux** steps above.

#### Native Windows Setup:

1. **Rust**: Download and run [rustup-init.exe](https://rustup.rs/).
2. **Node.js**: Download and install [Node.js v20+ LTS](https://nodejs.org/).
3. **Tesseract OCR**: Download installer from [UB-Mannheim Tesseract](https://github.com/UB-Mannheim/tesseract/wiki) and add `C:\Program Files\Tesseract-OCR` to your System `PATH`.
4. **Poppler for Windows**: Download Poppler binary archive, extract, and add its `bin/` folder to your System `PATH`.

---

## 3. Quick Setup & Running the Project

Extract your demo ZIP archive and navigate to the directory:

```bash
unzip document_scanner.zip -d document_scanner
cd document_scanner
```

---

### Option A: Standalone Mode (Recommended — Easiest!)

In this mode, the Rust backend compiles the frontend and serves **both the backend API and the full interactive UI on a single port (`8081`)**.

#### Step 1: Build Frontend

```bash
cd frontend
npm install
npm run build
cd ..
```

#### Step 2: Run the Rust Server

```bash
cargo run --release
```

#### Step 3: Open in Browser

Visit: **[http://localhost:8081](http://localhost:8081)**

_(That's it! Everything works from one single process)._

---

### Option B: Development Mode (with Live Reload)

Use this option if you want to modify code and see changes in real time.

#### Terminal 1 — Start Rust Backend:

```bash
cargo run
```

_Backend runs on `http://localhost:8081`._

#### Terminal 2 — Start Vue 3 Vite Dev Server:

```bash
cd frontend
npm install
npm run dev
```

_Frontend runs on `http://localhost:5173` (or `5174`) and automatically proxies all `/api` requests to the Rust backend on port `8081`._

Open your browser at: **[http://localhost:5173](http://localhost:5173)**

---

### Option C: Docker (Zero Local Dependencies)

If you have Docker installed, you don't need to install Rust, Node, or Tesseract on your host machine.

```bash
# 1. Build the production Docker container
docker build -t document_scanner .

# 2. Run the container
docker run -p 8081:8081 --name scanner_app document_scanner
```

Open: **[http://localhost:8081](http://localhost:8081)**

---

## 4. How to Verify the Installation

To verify that the backend engine and all system dependencies are healthy:

### 1. Check Health Endpoint

Run in terminal:

```bash
curl http://localhost:8081/api/health
```

Expected JSON response:

```json
{
  "service": "rust-document-scanner",
  "version": "0.1.0",
  "status": "healthy",
  "tesseract_path": "/usr/bin/tesseract",
  "max_file_size_mb": 35
}
```

If `"status": "healthy"` appears, OCR and system paths are correctly configured!

---

## 5. Testing Features

Once the app is running in your browser:

1. **Upload or Drop a File**:
   - Drag and drop any **Delivery Challan**, **Tax Invoice**, **Bill of Supply**, or receipt (PDF, JPG, PNG, WEBP).
   - Click **"Scan & Extract Document"**.

2. **Verify Extracted Fields**:
   - **Document Type**: Detected automatically (Tax Invoice, Delivery Challan, etc.).
   - **Voucher / Invoice Number**: Extracted into the field.
   - **Date & Party Details**: Extracted Party name, GSTIN, and Address.
   - **Items Grid**: Product descriptions, HSN, Quantities, Unit, Rates, and Amounts.
   - **Totals & Taxes**: Subtotal, CGST, SGST, IGST, and Grand Total.

3. **Interactive Raw OCR Inspector (Hover-to-Copy)**:
   - Scroll down to the **Raw OCR Text** tab.
   - Hover your mouse over any word:
     - An interactive floating popover appears.
     - Click **📋 Copy Word** to copy the specific word.
     - Click **💬 Copy Sentence** to copy the whole sentence.
     - Click **📄 Copy Line** to copy the entire line.
   - A modern toast confirmation appears in the bottom right corner.

---

## 6. Troubleshooting & Common Issues

### Issue 1: `node_modules/vite/bin/vite.js: SyntaxError: Unexpected token '.'`

- **Cause**: The active Node.js version is older than v18/v20 (e.g. Node 12 or 14).
- **Solution**: Upgrade or switch to Node 20 or 22:
  ```bash
  nvm install 20 && nvm use 20
  node -v
  ```

### Issue 2: `pdftotext` or `pdftoppm` command not found

- **Cause**: Missing Poppler utilities for PDF parsing.
- **Solution**: Install `poppler-utils`:

  ```bash
  # Debian/Ubuntu:
  sudo apt install -y poppler-utils

  # macOS:
  brew install poppler
  ```

### Issue 3: `Tesseract binary not found at /usr/bin/tesseract`

- **Cause**: Tesseract is installed in a different directory or not installed.
- **Solution**:
  1. Check where Tesseract is installed: `which tesseract`
  2. If located at e.g. `/usr/local/bin/tesseract`, pass it via environment variable or in `.env`:
     ```bash
     export TESSERACT_PATH=$(which tesseract)
     cargo run
     ```

### Issue 4: `Address already in use (os error 98)`

- **Cause**: Port `8081` is already being used by another process.
- **Solution**: Either stop the other process or set a custom port:
  ```bash
  export PORT=8085
  cargo run
  ```
