# rewrit - Setup Guide

## Prerequisites

### 1. Install Rust

```bash
# macOS/Linux
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh

# Follow the prompts and restart your terminal
# Verify installation
rustc --version
cargo --version
```

### 2. Install Tauri Prerequisites (macOS)

```bash
xcode-select --install
```

### 3. Initialize Tauri

```bash
npm install --save-dev @tauri-apps/cli
npm run tauri init
```

When prompted:
- App name: `rewrit`
- Window title: `rewrit`
- Web assets location: `../dist`
- Dev server URL: `http://localhost:5173`
- Frontend dev command: `npm run dev`
- Frontend build command: `npm run build`

## Project Structure

```
rewrit/
├── src/                    # Svelte frontend
│   ├── lib/               # Svelte components
│   ├── stores/            # State management
│   └── App.svelte         # Main app component
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   ├── device_id.rs   # Device ID management
│   │   ├── hotkey.rs      # Global hotkey handling
│   │   ├── clipboard.rs   # Clipboard operations
│   │   └── api_client.rs  # Backend API communication
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── backend/               # FastAPI backend
│   ├── main.py
│   ├── models.py
│   ├── database.py
│   └── requirements.txt
└── DoD.md                 # Specifications

```

## Development

### Frontend (Svelte + Vite)
```bash
npm run dev
```

### Tauri (Desktop App)
```bash
npm run tauri dev
```

### Backend (FastAPI)
```bash
cd backend
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install -r requirements.txt
uvicorn main:app --reload
```

## Build

```bash
# macOS
npm run tauri build -- --target universal-apple-darwin

# Windows
npm run tauri build -- --target x86_64-pc-windows-msvc
```

## Next Steps

1. Install Rust (see above)
2. Run `npm run tauri init`
3. Start development with `npm run tauri dev`
