# rewrit - AI-powered text transformation

> "L'IA au bout de tes doigts" - Instant, frictionless text transformation

Transform text instantly with global shortcuts, powered by Groq AI. Translate, rephrase, and transform text without leaving your current application.

## Features (MVP 1)

- ✨ **Global Shortcuts**: `Cmd+Alt+T` (EN→FR) / `Cmd+Alt+Y` (FR→EN) on Mac
- 🔄 **Instant Translation**: EN ↔ FR
- 📊 **Quota System**: 5 free actions per day
- 💾 **Anonymous**: No email or password required
- 🚀 **Fast**: < 3 seconds translation time
- 🎨 **Beautiful UI**: Native desktop app with modern interface

## Tech Stack

**Frontend:**
- Svelte 4
- TailwindCSS
- Vite
- Lucide Icons

**Backend (Desktop App):**
- Tauri 1.5+
- Rust 1.70+
- tokio (async runtime)
- reqwest (HTTP client)

**Backend (API):**
- FastAPI (Python 3.11+)
- PostgreSQL
- SQLAlchemy
- Groq API (LLaMA 3.3)

## Prerequisites

### 1. Install Rust

```bash
# macOS/Linux
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh

# Restart terminal and verify
rustc --version
cargo --version
```

### 2. Install Node.js

Download from [nodejs.org](https://nodejs.org/) (LTS version recommended)

### 3. Install Python 3.11+

Download from [python.org](https://www.python.org/downloads/)

### 4. Install PostgreSQL

```bash
# macOS
brew install postgresql@15
brew services start postgresql@15

# Windows
# Download from https://www.postgresql.org/download/windows/
```

## Setup Instructions

### 1. Clone and Install Dependencies

```bash
cd rewrit

# Install frontend dependencies
npm install

# Initialize Tauri (if not already done)
npm install --save-dev @tauri-apps/cli
```

### 2. Set up Backend API

```bash
cd backend

# Create virtual environment
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt

# Create .env file
cp .env.example .env
```

### 3. Configure Environment

Edit `backend/.env`:

```env
DATABASE_URL=postgresql://localhost:5432/rewrit
ENVIRONMENT=development
PORT=8000
```

AI translation is handled directly by the Tauri desktop app via the Groq API. The backend only manages quota and device registration.

### 4. Set up Database

```bash
# Create database
createdb rewrit

# Run the app once to create tables (they're created automatically)
# Or use Alembic for migrations
```

## Development

### Quick Start (macOS - Recommended)

Launch everything automatically:

```bash
./dev-start.sh
```

This script will:
- ✅ Check all prerequisites
- ✅ Install missing dependencies
- ✅ Create database if needed
- ✅ Launch backend and frontend in separate terminals
- ✅ Display current configuration (AI or Mock mode)

### Manual Start

#### Run Backend API

```bash
cd backend
source venv/bin/activate
python main.py

# Or with uvicorn
uvicorn main:app --reload
```

The API will be available at `http://localhost:8000`

#### Run Tauri App

```bash
# From project root
npm run tauri:dev
```

This will:
1. Start the Vite dev server (frontend)
2. Build and run the Tauri app (Rust backend + Svelte UI)

### Check Current Mode

```bash
./check-mode.sh
```

Shows whether you're in AI mode or Mock mode

## Usage

1. **Start the app**: The rewrit icon will appear in your system tray
2. **Select text** in any application
3. **Press** `Cmd+Alt+T` (EN→FR) or `Cmd+Alt+Y` (FR→EN) on Mac
4. **Text is automatically translated** and pasted back

### Settings Window

- **Right-click** the system tray icon → Settings
- Or press `Cmd+,` (Mac)

Features:
- View quota usage (X/5 actions today)
- See time until quota reset
- Toggle rewrit on/off
- Copy Device ID for support

## Building for Production

### macOS

```bash
npm run tauri build -- --target universal-apple-darwin
```

Output: `src-tauri/target/release/bundle/dmg/rewrit_1.0.0_universal.dmg`

### Windows

```bash
npm run tauri build -- --target x86_64-pc-windows-msvc
```

Output: `src-tauri/target/release/bundle/msi/rewrit_1.0.0_x64_en-US.msi`

## Project Structure

```
rewrit/
├── src/                          # Svelte frontend
│   ├── lib/
│   │   ├── components/          # UI components
│   │   │   ├── Toast.svelte
│   │   │   ├── Settings.svelte
│   │   │   └── QuotaExceededModal.svelte
│   │   └── stores/              # State management
│   │       ├── quota.js
│   │       ├── toast.js
│   │       └── settings.js
│   ├── App.svelte               # Main app
│   └── main.js                  # Entry point
│
├── src-tauri/                   # Rust backend
│   ├── src/
│   │   ├── main.rs              # Tauri app entry point
│   │   ├── device_id.rs         # Device ID management
│   │   └── api_client.rs        # API communication
│   ├── Cargo.toml               # Rust dependencies
│   └── tauri.conf.json          # Tauri configuration
│
├── backend/                     # FastAPI backend
│   ├── main.py                  # API endpoints
│   ├── models.py                # Database models
│   ├── database.py              # Database connection
│   ├── requirements.txt         # Python dependencies
│   └── .env                     # Environment variables
│
├── DoD.md                       # Specifications (French)
├── SETUP.md                     # Setup guide
└── README.md                    # This file
```

## API Endpoints

### POST /api/v1/register
Register a new device

### GET /api/v1/quota
Get quota information

### POST /api/v1/translate
Translate text

See `DoD.md` for detailed API specifications.

## Troubleshooting

### "Rust not found" error
- Make sure Rust is installed: `rustc --version`
- Restart your terminal after installing Rust

### "Database connection error"
- Check PostgreSQL is running: `brew services list` (macOS)
- Verify DATABASE_URL in `.env`
- Create database: `createdb rewrit`

### Global shortcut not working (macOS)
- Grant Accessibility permissions to rewrit
- System Settings → Privacy & Security → Accessibility

### Text replacement not working
- Make sure text is selected before pressing the shortcut
- Check that rewrit is enabled in Settings
- Verify clipboard permissions

## Roadmap (Post-MVP)

### Phase 2 (Weeks 5-8)
- [ ] Multi-actions (translate, rephrase, fix, summarize, expand)
- [ ] Preview before replacement
- [ ] Custom prompts (1 slot)
- [ ] Stripe integration + Pro subscription
- [ ] Web dashboard
- [ ] Email notifications

### Future
- [ ] Multi-language support (ES, DE, IT, PT, etc.)
- [ ] Custom keyboard shortcuts
- [ ] Action history
- [ ] Offline mode
- [ ] Auto-update
- [ ] Cross-device sync

## Contributing

This is currently a solo project for MVP 1. Contributions will be welcome after the initial release.

## License

All rights reserved. License TBD.

## Support

For issues or questions:
- Email: [your-email]
- GitHub Issues: [repo-url]

Include your Device ID (found in Settings) when reporting issues.

---

**Made with ❤️ by Yvan**

*rewrit - L'IA au bout de tes doigts*
