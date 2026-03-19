# rewrit

> **L'IA au bout de tes doigts** — Instant, frictionless text transformation

Transform text instantly via global keyboard shortcuts without leaving your current application. Select text, press a shortcut, get the result. Translate, rephrase, summarize — powered by your choice of AI provider.

![macOS](https://img.shields.io/badge/macOS-supported-green) ![Windows](https://img.shields.io/badge/Windows-supported-green) ![Linux](https://img.shields.io/badge/Linux-supported-green)

## How it works

1. **Select text** in any app
2. **Press a shortcut** (e.g. `Cmd+Alt+T`)
3. **Text is transformed** and pasted back instantly

That's it. No copy-paste into a browser. No switching apps. No waiting.

## Features

- **Global shortcuts** — Works in any application (Notes, Word, Slack, VS Code, browser...)
- **Multi-provider** — Choose your AI: Groq, Mistral, Gemini, or Ollama (local/offline)
- **BYOK** — Bring Your Own Key, use your free tier or paid account
- **Quick Select** — Pick language + tone via keyboard (`Cmd+Shift+Space`)
- **Preview mode** — See the result before pasting, with streaming text display
- **Direct paste mode** — Instant replacement with progress bar
- **Streaming** — Watch the translation appear in real-time
- **Custom shortcuts** — Create your own shortcuts with custom prompts
- **System tray** — Runs quietly in the background
- **Configurable** — All settings persisted between sessions

## AI Providers

| Provider | Speed | Free tier | Data location |
|----------|-------|-----------|---------------|
| **Groq** | ~300ms | 1000 req/day | USA |
| **Mistral** | ~600ms | 1 req/s | EU (France) |
| **Gemini** | ~2.5s | 15 req/min | USA |
| **Ollama** | ~2-25s | Unlimited (local) | Your machine |

## Download

Go to [Releases](https://github.com/YvanBetremieux/rewrit/releases) and download:

- **macOS Apple Silicon**: `rewrit_*_aarch64.dmg`
- **macOS Intel**: `rewrit_*_x64.dmg`
- **Windows**: `rewrit_*_x64-setup.exe`
- **Linux**: `rewrit_*_amd64.deb` or `.AppImage`

### First launch (macOS)

The app is not signed with an Apple Developer certificate. macOS will block it by default. Run this once after installing:

```bash
xattr -cr /Applications/rewrit.app
```

Then macOS will ask for **Accessibility permissions** — this is required to capture keyboard shortcuts and simulate copy/paste. Go to **System Settings > Privacy & Security > Accessibility** and enable rewrit.

## Setup

1. Launch rewrit — it appears in your system tray
2. Right-click the tray icon > **Settings**
3. Choose your **AI Provider** (Groq recommended for speed)
4. Enter your **API key** (click the link to create one for free)
5. Start using shortcuts!

### Default shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+Alt+T` | Translate EN → FR |
| `Cmd+Alt+Y` | Translate FR → EN |
| `Cmd+Shift+Space` | Quick Select (language + tone picker) |

All shortcuts are customizable in Settings.

## Tech Stack

| Component | Technology |
|-----------|------------|
| Desktop app | [Tauri 2](https://v2.tauri.app/) (Rust) |
| Frontend | [Svelte 4](https://svelte.dev/) + [TailwindCSS](https://tailwindcss.com/) |
| AI inference | Groq / Mistral / Gemini API, Ollama (local) |
| Backend API | [FastAPI](https://fastapi.tiangolo.com/) (Python) + PostgreSQL |

## Development

### Prerequisites

- Rust 1.70+ (`curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh`)
- Node.js 18+ ([nodejs.org](https://nodejs.org/))
- Python 3.11+ (for backend)
- PostgreSQL 15+ (for backend)

### Quick start

```bash
# Install dependencies
npm install

# Start dev (launches Vite + Tauri)
npm run tauri:dev
```

Or use the automated script:

```bash
./dev-start.sh
```

### Project structure

```
rewrit/
├── src/                        # Svelte frontend
│   ├── lib/components/         # UI components
│   │   ├── Settings.svelte     # Settings window (providers, shortcuts)
│   │   ├── Preview.svelte      # Translation preview with streaming
│   │   ├── Progress.svelte     # Progress indicator (direct paste)
│   │   └── QuickSelect.svelte  # Language + tone picker
│   └── lib/stores/             # Svelte state management
│
├── src-tauri/src/              # Rust backend
│   ├── main.rs                 # App entry, shortcuts, system tray
│   ├── llm_client.rs           # Multi-provider LLM client (Groq/Mistral/Gemini/Ollama)
│   ├── device_id.rs            # Device ID + settings persistence (~/.rewrit/)
│   ├── api_client.rs           # Backend API communication
│   └── config.rs               # Timing configuration
│
├── backend/                    # FastAPI backend (quota management)
│   ├── main.py
│   ├── models.py
│   └── database.py
│
├── .github/workflows/          # CI/CD
│   └── release.yml             # Auto-build on version tag
│
└── docs/                       # Documentation
```

### Building for production

```bash
# macOS (Apple Silicon)
npm run tauri build -- --target aarch64-apple-darwin

# macOS (Intel)
npm run tauri build -- --target x86_64-apple-darwin

# Or just push a version tag to trigger CI:
git tag v1.0.1 && git push origin v1.0.1
```

## Settings persistence

All settings are stored in `~/.rewrit/`:

| File | Description |
|------|-------------|
| `llm_provider` | Active provider (groq/mistral/gemini/ollama) |
| `groq_api_key` | Groq API key |
| `mistral_api_key` | Mistral API key |
| `gemini_api_key` | Gemini API key |
| `show_preview` | Preview mode on/off |
| `stream_response` | Streaming on/off |
| `quick_select_enabled` | Quick Select on/off |
| `quick_select_shortcut` | Quick Select shortcut |
| `device_id` | Anonymous device identifier |

## Troubleshooting

### Shortcut not working (macOS)
Grant Accessibility permissions: **System Settings > Privacy & Security > Accessibility** > enable rewrit.

### "Cannot connect to Ollama"
Make sure Ollama is running: `ollama serve`

### Translation fails
Check your API key in Settings. Try switching to another provider.

### App doesn't appear
Look for the rewrit icon in your **system tray** (menu bar on macOS).

## License

All rights reserved. License TBD.

---

**Made with care by [Yvan](https://github.com/YvanBetremieux)**

*rewrit — L'IA au bout de tes doigts*
