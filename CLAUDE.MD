# rewrit - Project Context for Claude

## Project Name
**rewrit** - AI-powered instant text transformation

## Description
Desktop application that transforms text instantly via global keyboard shortcuts without leaving the current application. Built as a Tauri desktop app (Rust + Svelte) with a FastAPI backend. AI transformations are handled client-side via Groq API (LLaMA 3.3).

**Tagline**: "L'IA au bout de tes doigts" - Instant, frictionless text transformation

**Goal**: Achieve financial independence (10k€/month MRR) through freemium SaaS model.

## Tech Stack

### Desktop App
- **Tauri 1.5+**: Cross-platform desktop framework
- **Rust 1.70+**: Backend logic, global shortcuts, clipboard management
- **Svelte 4**: Reactive UI framework
- **TailwindCSS**: Utility-first CSS
- **Lucide Icons**: Icon library
- **Vite**: Build tool

### AI
- **Groq API**: LLM inference (LLaMA 3.3 70B Versatile)
- AI calls happen client-side in Tauri, not in the backend

### Backend API
- **Python 3.11+**: Backend language
- **FastAPI**: Modern API framework
- **PostgreSQL**: Database (Railway managed)
- **SQLAlchemy**: ORM
- **Railway.app**: Hosting platform

## Project Structure

```
rewrit/
├── src/                          # Svelte Frontend
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
│   └── main.js
│
├── src-tauri/                   # Rust Backend
│   ├── src/
│   │   ├── main.rs              # Entry point, global shortcuts, system tray
│   │   ├── device_id.rs         # Device ID generation/storage
│   │   ├── api_client.rs        # Backend API communication
│   │   ├── groq_client.rs       # Groq AI API client
│   │   └── double_tap.rs        # Double-tap Ctrl detection (rdev)
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── backend/                     # FastAPI Backend
│   ├── main.py                  # API endpoints
│   ├── models.py                # Database models
│   ├── database.py              # DB connection
│   ├── config.py                # Configuration & feature flags
│   ├── requirements.txt
│   └── .env
│
├── docs/                        # Documentation (ALL new docs go here!)
│   ├── INDEX.md                 # Documentation index
│   ├── SETUP.md                 # Prerequisites & installation
│   ├── FEATURE_FLAGS.md         # Configuration (Groq, backend, settings)
│   ├── QUICK_START_FR.md        # Quick start guide (French)
│   ├── CHEAT_SHEET.md           # Commands & shortcuts reference
│   ├── NOUVELLES_FONCTIONNALITES.md  # New features summary
│   └── IMPLEMENTATION_SUMMARY.md     # Implementation details
│
├── DoD.md                       # Full specifications (French)
├── README.md                    # Main documentation
├── CLAUDE.MD                    # This file - project context
├── dev-start.sh                 # Automated dev environment launcher
└── quickstart.sh                # Initial setup script
```

### Documentation Guidelines

**IMPORTANT**: When creating new documentation files, ALWAYS place them in the `docs/` folder. This includes:
- Feature guides
- Setup instructions
- API documentation
- User guides
- Tutorial files
- Implementation summaries
- Any markdown files that document the project

The only exceptions are:
- `README.md` (stays at root for GitHub display)
- `CLAUDE.MD` (stays at root for AI context)
- `DoD.md` (stays at root as source of truth)

**When adding new documentation**:
1. Create the file in `docs/`
2. Update `docs/INDEX.md` with a link to the new document
3. If relevant, add a reference from README.md pointing to the new doc

## Getting Started

### Prerequisites
1. Rust (rustup)
2. Node.js & npm
3. Python 3.11+
4. PostgreSQL 15+

### Quick Start
```bash
# Run the quickstart script
./quickstart.sh

# Or manually:
# Terminal 1 - Backend
cd backend
source venv/bin/activate
python main.py

# Terminal 2 - Desktop App
npm run tauri:dev
```

## Architecture

### Key Components

1. **Tauri App (Rust)**
   - Global shortcut listeners (Cmd+Alt+T for EN→FR, Cmd+Alt+Y for FR→EN)
   - Quick Select (double-tap Ctrl) for language+tone picker
   - Clipboard capture and text replacement
   - Groq API client for AI translation
   - System tray integration
   - Device ID management
   - Backend API client for quota/registration

2. **Svelte Frontend**
   - Settings window UI
   - Quota tracking and display
   - Toast notifications
   - Modal dialogs

3. **FastAPI Backend**
   - Device registration
   - Quota management (5 actions/day free)
   - Usage tracking and logging
   - PostgreSQL database

### Core Workflow

1. User selects text in any app
2. Presses global shortcut (Cmd+Alt+T for EN→FR, Cmd+Alt+Y for FR→EN)
3. Tauri captures clipboard content via simulated Cmd+C
4. Calls Groq API directly for AI translation
5. Optionally shows preview window (if enabled in settings)
6. Pastes translated text back via simulated Cmd+V

### Design Decisions

- **Device ID instead of auth**: Simplifies MVP, anonymous usage
- **Clipboard-based**: Cross-platform text replacement
- **Freemium quota**: 5 free actions/day, reset at midnight UTC
- **Tauri over Electron**: Smaller bundle size, better performance
- **Groq + LLaMA**: Fast inference, client-side API calls
- **PostgreSQL**: Reliable, Railway-hosted for easy deployment

## Development Guidelines

### Code Style
- **Rust**: Follow `cargo fmt` conventions
- **Svelte**: Reactive statements, composition over inheritance
- **Python**: PEP 8, type hints preferred
- **CSS**: Utility-first with Tailwind

### Testing
- Manual testing checklist in IMPLEMENTATION_SUMMARY.md
- Test translation in multiple apps (Notes, Word, Browser, etc.)
- Verify quota tracking accuracy
- Test on both macOS and Windows

### Git Workflow
- Main branch for stable releases
- Feature branches for new features
- Commit messages: descriptive, present tense

## Known Issues

1. **macOS Accessibility**: User must grant permissions manually
2. **Text replacement**: Limited to clipboard-compatible apps
3. **Quota bypass**: Users can reinstall for new device_id (acceptable for MVP1)
4. **First build**: Rust compilation takes 5-10 minutes

## Future Enhancements

### Phase 2 (Weeks 5-8)
- Multiple actions (translate, rephrase, fix, summarize, expand)
- Preview before replacement
- Custom prompts (1 slot)
- Stripe payment integration
- Web dashboard
- Email notifications

### Later Phases
- Multi-language support (ES, DE, IT, PT)
- Custom keyboard shortcuts
- Action history
- Offline mode
- Auto-update
- Cross-device sync

## Additional Context for Claude

### Important Notes

1. **DoD.md is the source of truth**: All specifications in French
2. **MVP 1 only**: Don't implement features marked ❌ in DoD.md
3. **No over-engineering**: Keep it simple, follow specifications exactly
4. **Cost-conscious**: Use Groq free tier, optimize API calls
5. **User experience first**: <3s latency target, clear error messages

### Deprecated Patterns to Avoid
- Don't use email/password authentication (use device_id)
- Don't implement payment in MVP1 (placeholder only)
- Don't create custom LLM prompts yet (fixed translation only)

### Unconventional Approaches
- Anonymous users via device_id (UUID v4)
- Clipboard-based text replacement (no direct text editing)
- Daily quota reset via database query (no cron job in MVP1)

### Common Tasks

**Add a new Tauri command:**
```rust
// In src-tauri/src/main.rs
#[tauri::command]
async fn my_command() -> Result<String, String> {
    Ok("result".to_string())
}

// Add to invoke_handler
.invoke_handler(tauri::generate_handler![
    get_device_id,
    fetch_quota,
    translate_text,
    my_command  // Add here
])
```

**Add a new API endpoint:**
```python
# In backend/main.py
@app.post("/api/v1/my-endpoint")
async def my_endpoint(
    request: MyRequest,
    x_device_id: str = Header(..., alias="X-Device-ID"),
    db: Session = Depends(get_db)
):
    # Implementation
    return response
```

**Update Svelte store:**
```javascript
// In src/lib/stores/mystore.js
import { writable } from 'svelte/store';

export const myStore = writable(initialValue);

export function updateMyStore(newValue) {
    myStore.set(newValue);
}
```

## Current Status

✅ **MVP 1 Implementation: COMPLETE**

All core features implemented:
- Global shortcut functionality
- Translation EN ↔ FR
- Quota system (5/day)
- Settings UI
- Toast notifications
- Backend API
- Database models
- Groq AI integration

**Next Step**: Install Rust, configure environment, and run!

---

**For detailed specifications**, see: DoD.md
**For setup instructions**, see: README.md
**For implementation details**, see: docs/IMPLEMENTATION_SUMMARY.md
**For all documentation**, see: docs/INDEX.md
