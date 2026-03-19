# rewrit - Architecture technique

## Table des matieres

1. [Vue d'ensemble](#vue-densemble)
2. [Point d'entree et initialisation](#point-dentree-et-initialisation)
3. [Architecture des threads](#architecture-des-threads)
4. [Les 3 fenetres](#les-3-fenetres)
5. [Scenario 1 : Raccourci clavier (Cmd+Alt+T/Y)](#scenario-1--raccourci-clavier-cmdaltty)
6. [Scenario 2 : Quick Select (double-tap Ctrl)](#scenario-2--quick-select-double-tap-ctrl)
7. [Scenario 3 : Settings (System Tray)](#scenario-3--settings-system-tray)
8. [Communication Rust ↔ Svelte](#communication-rust--svelte)
9. [State machine du double-tap](#state-machine-du-double-tap)
10. [AppState : etat partage](#appstate--etat-partage)
11. [Clients externes](#clients-externes)
12. [Arbre d'appels complet](#arbre-dappels-complet)

---

## Vue d'ensemble

```
┌──────────────────────────────────────────────────────┐
│                    macOS / Windows                     │
│                                                       │
│   ┌─────────┐    ┌─────────────────────────────────┐ │
│   │  User    │    │          rewrit (Tauri)          │ │
│   │  selects ├───►│                                  │ │
│   │  text    │    │  ┌──────────┐  ┌──────────────┐ │ │
│   └─────────┘    │  │  rdev    │  │ global_short  │ │ │
│                   │  │  thread  │  │ cut handler   │ │ │
│                   │  │(Ctrl×2) │  │ (Cmd+Alt+T/Y)│ │ │
│                   │  └────┬─────┘  └──────┬───────┘ │ │
│                   │       │               │          │ │
│                   │       ▼               ▼          │ │
│                   │  ┌──────────────────────────┐    │ │
│                   │  │     AppState (Mutex)      │    │ │
│                   │  │  original_text, prompt,   │    │ │
│                   │  │  is_processing, etc.      │    │ │
│                   │  └────────────┬──────────────┘    │ │
│                   │               │                   │ │
│                   │               ▼                   │ │
│                   │  ┌──────────────────────────┐    │ │
│                   │  │   GroqClient (async)      │    │ │
│                   │  │   Groq API → LLaMA 3.3    │    │ │
│                   │  └────────────┬──────────────┘    │ │
│                   │               │                   │ │
│                   │     ┌─────────┴─────────┐        │ │
│                   │     ▼                   ▼        │ │
│                   │  Preview window    Direct paste   │ │
│                   │  (Svelte)          (Cmd+V)       │ │
│                   └──────────────────────────────────┘ │
│                                                       │
│   ┌─────────────────────────────────────────────────┐ │
│   │          Backend FastAPI (localhost:8000)         │ │
│   │     device registration, quota tracking          │ │
│   └─────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────┘
```

L'application est un **tray app** (pas de fenetre principale visible au lancement). Tout part du `system tray` ou des `raccourcis globaux`.

---

## Point d'entree et initialisation

**Fichier** : `src-tauri/src/main.rs`

```
main() → run()
```

### `run()` - Sequence d'initialisation

```
run()
 ├── dotenv::dotenv()                    // Charge .env (GROQ_API_KEY)
 ├── device_id::get_device_id()          // Lit ou genere ~/.rewrit/device_id (UUID v4)
 ├── ApiClient::new(device_id)           // Client HTTP pour le backend FastAPI
 ├── Arc<AtomicBool>::new(true)          // Flag atomique pour Quick Select
 ├── AppState { ... }                    // Construction du state partage
 │
 └── tauri::Builder::default()
      ├── .plugin(clipboard_manager)
      ├── .plugin(dialog)
      ├── .plugin(shell)
      ├── .manage(app_state)             // Injecte AppState dans Tauri
      ├── .invoke_handler([...])         // Enregistre 16 commandes Tauri
      │
      └── .setup(|app| {
           ├── set_activation_policy(Accessory)  // macOS: tray-only, pas dans le Dock
           ├── osascript warm-up                  // Pre-charge osascript (evite latence)
           │
           ├── global_shortcut::Builder           // Enregistre le handler de raccourcis
           │    └── with_handler(handle_global_shortcut)
           │
           ├── TrayIconBuilder                    // Cree l'icone system tray
           │    ├── menu: [Settings, Quit]
           │    └── on_menu_event: { settings → show main window, quit → exit }
           │
           ├── Register default shortcuts         // Cmd+Alt+Y (FR→EN), Cmd+Alt+T (EN→FR)
           │    └── shortcuts HashMap<id, ShortcutConfig>
           │
           ├── Pre-create hidden windows          // Preview + QuickSelect (warm-up Vite)
           │    ├── WebviewWindow("preview")      // 650x450, hidden
           │    └── WebviewWindow("quickselect")  // 340x280, hidden
           │
           ├── double_tap::start_listener()       // Lance le thread rdev
           │
           ├── spawn → init_app()                 // Init GroqClient avec GROQ_API_KEY
           └── spawn → client.register()          // Enregistrement backend (non-bloquant)
      })
      └── .run()                                  // Demarre la boucle evenementielle Tauri
```

---

## Architecture des threads

```
┌─────────────────────────────────────────────────────────────┐
│                        Processus rewrit                      │
│                                                              │
│  ┌────────────────────┐                                      │
│  │   Main Thread       │  Boucle Tauri, UI, tray menu       │
│  │   (Tauri event      │  Gere les commandes invoke          │
│  │    loop)            │                                     │
│  └────────────────────┘                                      │
│                                                              │
│  ┌────────────────────┐                                      │
│  │   Global Shortcut   │  Thread dedie par tauri-plugin      │
│  │   Handler Thread    │  Appelle handle_global_shortcut()   │
│  └────────────────────┘  (synchrone, peut block_on)          │
│                                                              │
│  ┌────────────────────┐                                      │
│  │   rdev Listener     │  std::thread::spawn                 │
│  │   Thread            │  rdev::listen() bloque ce thread    │
│  └────────────────────┘  Detecte double-tap Ctrl             │
│           │                                                  │
│           │ double-tap detecte                               │
│           ▼                                                  │
│  ┌────────────────────┐                                      │
│  │   Quick Select      │  std::thread::spawn (ephemere)     │
│  │   Handler Thread    │  handle_quick_select_trigger()      │
│  └────────────────────┘  (synchrone, peut block_on)          │
│                                                              │
│  ┌────────────────────┐                                      │
│  │   Tokio Runtime     │  tauri::async_runtime::spawn        │
│  │   (thread pool)     │  GroqClient, ApiClient, clipboard   │
│  └────────────────────┘  restore                             │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**Point critique** : `handle_global_shortcut()` et `handle_quick_select_trigger()` utilisent `tauri::async_runtime::block_on()` pour appeler osascript. Ils doivent tourner sur un thread standard (pas tokio) sinon panic "Cannot start a runtime from within a runtime".

---

## Les 3 fenetres

| Fenetre | Label Tauri | Taille | Fichiers | Role |
|---------|-------------|--------|----------|------|
| **Settings** | `"main"` | 800x600 | `index.html` → `App.svelte` → `Settings.svelte` | Configuration, quota, shortcuts |
| **Preview** | `"preview"` | 650x450 | `src/preview.html` → `Preview.svelte` | Affiche original vs traduit |
| **Quick Select** | `"quickselect"` | 340x280 | `src/quickselect.html` → `QuickSelect.svelte` | Picker langue + ton |

Toutes sont `decorations: false`, `always_on_top: true`, `skip_taskbar: true`.
Preview et QuickSelect sont pre-creees au demarrage (hidden) pour eviter la latence de compilation Vite.

---

## Scenario 1 : Raccourci clavier (Cmd+Alt+T/Y)

### Diagramme de sequence

```
 User        OS        GlobalShortcut     handle_global_    GroqClient    Preview
  │           │         Handler Thread     shortcut()                     Window
  │           │              │                 │                │           │
  │  Select   │              │                 │                │           │
  │  text     │              │                 │                │           │
  │───────────►              │                 │                │           │
  │           │              │                 │                │           │
  │  Cmd+Alt+T│              │                 │                │           │
  │───────────►──────────────►                 │                │           │
  │           │              │                 │                │           │
  │           │              │  find_prompt()  │                │           │
  │           │              │────────────────►│                │           │
  │           │              │                 │                │           │
  │           │              │                 │ is_processing  │           │
  │           │              │                 │ = true         │           │
  │           │              │                 │                │           │
  │           │              │                 │ Enigo: release │           │
  │           │              │                 │ modifiers      │           │
  │           │              │                 │                │           │
  │           │              │                 │ clear clipboard│           │
  │           │              │                 │                │           │
  │           │              │                 │ osascript:     │           │
  │           │              │                 │ get frontmost  │           │
  │           │              │                 │ app name       │           │
  │           │              │                 │ → previous_app │           │
  │           │              │                 │                │           │
  │           │   Cmd+C      │                 │                │           │
  │           │◄─────────────┤─────────────────┤                │           │
  │           │              │                 │                │           │
  │           │              │                 │ read clipboard │           │
  │           │              │                 │ (8 retries)    │           │
  │           │              │                 │ → original_text│           │
  │           │              │                 │                │           │
  │           │              │       ┌─────────┴──────────┐     │           │
  │           │              │       │  show_preview?      │     │           │
  │           │              │       └──┬──────────────┬──┘     │           │
  │           │              │          │ true         │ false  │           │
  │           │              │          ▼              ▼        │           │
  │           │              │    Show Preview    Direct Paste  │           │
  │           │              │    (see below)     (see below)   │           │
```

### Branche Preview (`show_preview = true`)

```
  handle_global_shortcut()          Tokio                 Preview.svelte
         │                           │                         │
         │  window.show()            │                         │
         │──────────────────────────────────────────────────────►
         │                           │                         │
         │  emit("translation-      │                         │
         │       started")           │                         │
         │──────────────────────────────────────────────────────►
         │                           │                    isLoading=true
         │                           │                    show spinner
         │                           │                         │
         │  spawn async {            │                         │
         │──────────────────────────►│                         │
         │                           │                         │
         │                    groq.execute_prompt()             │
         │                           │──────► Groq API         │
         │                           │◄────── response         │
         │                           │                         │
         │                    pending_translation = result      │
         │                           │                         │
         │                    emit("show-preview",             │
         │                     {original, translated})         │
         │                           │─────────────────────────►
         │                           │                    isLoading=false
         │                           │                    show texts
         │                           │                         │
         │                    report_usage_to_backend()         │
         │                    is_processing = false             │
         │                           │                         │
```

### Interactions utilisateur dans la Preview

```
  Preview.svelte        Tauri Command           Effet
       │                     │                    │
       │  Enter              │                    │
       │──► handlePaste()    │                    │
       │    invoke("execute_paste")               │
       │─────────────────────►                    │
       │                     │  clipboard = translated
       │                     │  hide preview
       │                     │  refocus previous_app
       │                     │  osascript Cmd+V ──────► paste dans l'app
       │                     │  spawn: restore clipboard (500ms)
       │                     │                    │
       │  Space              │                    │
       │──► handleRetry()    │                    │
       │    invoke("retry_translation")           │
       │─────────────────────►                    │
       │                     │  spawn: re-appel Groq avec meme texte+prompt
       │                     │  emit "show-preview" avec nouveau resultat
       │                     │                    │
       │  Escape             │                    │
       │──► handleCancel()   │                    │
       │    invoke("cancel_preview")              │
       │─────────────────────►                    │
       │                     │  pending = None
       │                     │  hide preview
       │                     │  refocus previous_app
```

### Branche Direct Paste (`show_preview = false`)

```
  handle_global_shortcut()          Tokio                    App cible
         │                           │                         │
         │  spawn async {            │                         │
         │──────────────────────────►│                         │
         │                           │                         │
         │                    groq.execute_prompt()             │
         │                           │──────► Groq API         │
         │                           │◄────── response         │
         │                           │                         │
         │                    clipboard = translated            │
         │                    sleep(clipboard_ready)            │
         │                           │                         │
         │                    osascript Cmd+V                   │
         │                           │─────────────────────────►
         │                           │                    texte colle
         │                           │                         │
         │                    report_usage()                    │
         │                    is_processing = false             │
```

**Difference cle** : en mode direct, le texte original est **coupe** (`Cmd+X`) au lieu de copie. En preview, il est copie (`Cmd+C`) pour pouvoir annuler.

---

## Scenario 2 : Quick Select (double-tap Ctrl)

### Diagramme d'activite complet

```
  User          rdev Thread         DoubleTapDetector    New Thread     QuickSelect
   │                │                      │                │           .svelte
   │                │                      │                │              │
   │  Ctrl down     │                      │                │              │
   │───────────────►│  handle_event()      │                │              │
   │                │─────────────────────►│                │              │
   │                │                 Idle→FirstDown        │              │
   │                │                      │                │              │
   │  Ctrl up       │                      │                │              │
   │  (<200ms)      │                      │                │              │
   │───────────────►│─────────────────────►│                │              │
   │                │             FirstDown→FirstUp         │              │
   │                │                      │                │              │
   │  Ctrl down     │                      │                │              │
   │  (<400ms)      │                      │                │              │
   │───────────────►│─────────────────────►│                │              │
   │                │              FirstUp→SecondDown       │              │
   │                │                      │                │              │
   │  Ctrl up       │                      │                │              │
   │  (<200ms)      │                      │                │              │
   │───────────────►│─────────────────────►│                │              │
   │                │              SecondDown→DETECTED!     │              │
   │                │              reset → Idle             │              │
   │                │                      │                │              │
   │                │  std::thread::spawn ─────────────────►│              │
   │                │                      │                │              │
   │                │                      │    handle_quick_select_       │
   │                │                      │    trigger()   │              │
   │                │                      │                │              │
   │                │                      │       Enigo: release mods    │
   │                │                      │       clear clipboard        │
   │                │                      │       osascript: get app     │
   │                │                      │       osascript: Cmd+C       │
   │                │                      │       read clipboard         │
   │                │                      │       → original_text        │
   │                │                      │                │              │
   │                │                      │       window.show()          │
   │                │                      │       ──────────────────────►│
   │                │                      │       emit("quick-select-   │
   │                │                      │             open")           │
   │                │                      │       ──────────────────────►│
   │                │                      │       is_processing=false   reset()
   │                │                      │                │              │
   │  "2" (English) │                      │                │              │
   │────────────────┼──────────────────────┼────────────────┼─────────────►│
   │                │                      │                │    selectLanguage(en)
   │                │                      │                │              │
   │  "A" (Pro)     │                      │                │              │
   │────────────────┼──────────────────────┼────────────────┼─────────────►│
   │                │                      │                │    selectTone(pro)
   │                │                      │                │    tryExecute()
   │                │                      │                │    invoke("execute_
   │                │                      │                │     quick_select",
   │                │                      │                │     {en, professional})
   │                │                      │                │              │
```

### `execute_quick_select()` - Suite du flot

```
  execute_quick_select(language, tone)
         │
         ├── hide quickselect window
         ├── build_quick_select_prompt(language, tone)
         │    ex: "Rewrite this text in English with a professional, formal tone"
         │
         └── show_preview?
              ├── true  → meme flot que Scenario 1 branche Preview
              └── false → meme flot que Scenario 1 branche Direct Paste
```

### Cas de rejet du double-tap

```
  Ctrl down → C down → Ctrl up        ✗  other_key_during_press = true
                                          (c'est un Ctrl+C, pas un tap)

  Ctrl down ──(300ms)── Ctrl up        ✗  duration > MAX_TAP_DURATION (200ms)
                                          (c'est un hold, pas un tap)

  Ctrl tap ────(500ms)──── Ctrl tap    ✗  gap > DOUBLE_TAP_WINDOW (400ms)
                                          (trop lent entre les deux taps)

  Double-tap ──(0.5s)── Double-tap     ✗  cooldown < COOLDOWN (1s)
                                          (evite les triggers accidentels)

  Feature disabled in Settings         ✗  AtomicBool.load() == false
                                          (check a chaque event, effet immediat)
```

---

## Scenario 3 : Settings (System Tray)

```
  User         System Tray       on_menu_event       Main Window
   │               │                  │                   │
   │  click tray   │                  │                   │
   │──────────────►│                  │                   │
   │               │                  │                   │
   │  "Settings"   │                  │                   │
   │──────────────►│─────────────────►│                   │
   │               │                  │                   │
   │               │                  │  get_webview_     │
   │               │                  │  window("main")   │
   │               │                  │                   │
   │               │                  │  Exists?          │
   │               │                  ├─ Yes: show()      │
   │               │                  │       set_focus()  │
   │               │                  │  ────────────────►│
   │               │                  │                   │
   │               │                  ├─ No: create new   │
   │               │                  │  WebviewWindow    │
   │               │                  │  ("main", url)    │
   │               │                  │  800x600, center  │
   │               │                  │  ────────────────►│
   │               │                  │                   │
   │               │                  │              App.svelte monte
   │               │                  │              ├── get_device_id()
   │               │                  │              ├── fetch_quota()
   │               │                  │              └── listen(events...)
   │               │                  │                   │
   │               │                  │              Settings.svelte
   │               │                  │              ├── loadShowPreview()
   │               │                  │              ├── loadQuickSelectEnabled()
   │               │                  │              └── loadShortcuts()
```

### Commandes Tauri invoquees par Settings.svelte

| Invoke | Retour | Effet |
|--------|--------|-------|
| `get_device_id` | `String` | Affiche le device ID |
| `fetch_quota` | `QuotaInfo` | Affiche usage/limit |
| `reset_quota` | `QuotaInfo` | Remet le quota a 0 (debug) |
| `get_show_preview` | `bool` | Etat du toggle preview |
| `set_show_preview(show)` | `()` | Change le mode preview/direct |
| `get_quick_select_enabled` | `bool` | Etat du toggle Quick Select |
| `set_quick_select_enabled(enabled)` | `()` | Active/desactive double-tap Ctrl |
| `get_shortcuts` | `Vec<ShortcutConfig>` | Liste des raccourcis |
| `add_shortcut(shortcut, prompt)` | `ShortcutConfig` | Cree + enregistre un raccourci |
| `remove_shortcut(id)` | `()` | Desenregistre + supprime |
| `update_shortcut(id, shortcut?, prompt?)` | `ShortcutConfig` | Modifie un raccourci |

---

## Communication Rust ↔ Svelte

### Commandes (Svelte → Rust)

Appel via `invoke('command_name', { args })`. Synchrone du point de vue du frontend (attend la `Result<T, String>`).

```
Frontend                          Backend (Rust)
   │                                  │
   │  invoke('execute_paste')         │
   │─────────────────────────────────►│  fn execute_paste(state, app)
   │◄─────────────────────────────────│  → Result<(), String>
   │                                  │
```

### Events (Rust → Svelte)

Emit via `app.emit("event-name", payload)` ou `window.emit(...)`. Le frontend ecoute avec `listen('event-name', callback)`.

| Event | Emetteur | Payload | Cible |
|-------|----------|---------|-------|
| `translation-started` | `handle_global_shortcut` | `()` | Preview.svelte, App.svelte |
| `show-preview` | async task (apres Groq) | `{original, translated}` | Preview.svelte |
| `translation-error` | async task (erreur Groq) | `{error}` | App.svelte |
| `quota-updated` | `report_usage_to_backend` | `QuotaInfo` | App.svelte |
| `quota-exceeded` | `report_usage_to_backend` | `()` | App.svelte |
| `translation-completed` | `report_usage_to_backend` | `{quota}` | App.svelte |
| `quick-select-open` | `handle_quick_select_trigger` | `()` | QuickSelect.svelte |

---

## State machine du double-tap

**Fichier** : `src-tauri/src/double_tap.rs`

```
                    ┌──────────┐
             ┌─────│   Idle   │◄────────────────────────────┐
             │     └──────────┘                              │
             │          │                                    │
             │     Ctrl Press                                │
             │          │                                    │
             │          ▼                                    │
             │     ┌──────────┐    hold > 200ms             │
             │     │FirstDown │────────────────────────────►│
             │     └──────────┘    or other key pressed      │
             │          │                                    │
             │     Ctrl Release                              │
             │     (< 200ms)                                 │
             │          │                                    │
             │          ▼                                    │
             │     ┌──────────┐    gap > 400ms              │
             │     │ FirstUp  │───────────────────┐         │
             │     └──────────┘                    │         │
             │          │                     restart as     │
             │     Ctrl Press                 FirstDown      │
             │     (< 400ms)                       │         │
             │          │                          ▼         │
             │          ▼                     ┌──────────┐   │
             │     ┌───────────┐              │FirstDown │   │
             │     │SecondDown │              └──────────┘   │
             │     └───────────┘                             │
             │          │                                    │
             │     Ctrl Release                              │
             │     (< 200ms)                                 │
             │          │                                    │
             │          ▼                                    │
             │     ┌───────────┐    cooldown < 1s           │
             │     │ DETECTED! │────────────────────────────►│
             │     └───────────┘                              │
             │          │                                     │
             │     cooldown OK                                │
             │          │                                     │
             │          ▼                                     │
             │   callback → spawn thread                      │
             │   → handle_quick_select_trigger()              │
             │                                                │
             └────────────────────────────────────────────────┘
                    (tous les cas d'echec → reset Idle)
```

### Constantes

| Constante | Valeur | Role |
|-----------|--------|------|
| `DOUBLE_TAP_WINDOW` | 400ms | Delai max entre les deux taps |
| `MAX_TAP_DURATION` | 200ms | Duree max d'un tap (au-dela = hold) |
| `COOLDOWN` | 1s | Delai minimum entre deux triggers |

---

## AppState : etat partage

```rust
struct AppState {
    // --- Clients ---
    api_client: Mutex<Option<ApiClient>>,       // Backend FastAPI (quota)
    groq_client: Mutex<Option<GroqClient>>,     // LLM (Groq / LLaMA 3.3)
    device_id: Mutex<String>,                   // UUID v4 persistant

    // --- Etat de la transformation en cours ---
    pending_translation: Mutex<Option<String>>,  // Texte traduit en attente de paste
    original_text: Mutex<Option<String>>,         // Texte original (pour retry/restore)
    last_prompt: Mutex<Option<String>>,           // Dernier prompt (pour retry)
    previous_app: Mutex<Option<String>>,          // Nom de l'app a refocus (osascript)

    // --- Preferences ---
    show_preview: Mutex<bool>,                   // true = preview, false = direct paste

    // --- Concurrence ---
    is_processing: Mutex<bool>,                  // Empeche les triggers simultanes

    // --- Shortcuts ---
    shortcuts: Mutex<HashMap<String, ShortcutConfig>>,

    // --- Quick Select ---
    quick_select_enabled: Mutex<bool>,            // Toggle depuis Settings
    quick_select_enabled_flag: Arc<AtomicBool>,   // Partage avec rdev thread (lock-free)
    quick_select_open: Mutex<bool>,               // Fenetre QS ouverte? (pour toggle)
}
```

**Pourquoi deux champs pour `quick_select_enabled`?**
- Le `Mutex<bool>` est lu par les commandes Tauri (thread principal)
- Le `Arc<AtomicBool>` est lu par le thread rdev (pas de Mutex, lock-free via `Ordering::Relaxed`)
- `set_quick_select_enabled` met a jour les deux en meme temps

---

## Clients externes

### GroqClient (`groq_client.rs`)

```
GroqClient::execute_prompt(text, prompt) → TranslateResponse
    │
    ├── System prompt: "You are a helpful text transformation assistant..."
    ├── User prompt: "Instruction: {prompt}\n\nText to transform:\n{text}"
    │
    ├── POST https://api.groq.com/openai/v1/chat/completions
    │    model: "llama-3.3-70b-versatile"
    │    temperature: 0.3
    │    max_tokens: 2000
    │    timeout: 15s
    │
    └── clean_response()
         ├── Trim leading/trailing newlines
         ├── Remove wrapping quotes
         └── Restore original whitespace (leading_ws + result + trailing_ws)
```

### ApiClient (`api_client.rs`)

```
Base URL: http://localhost:8000/api/v1
Headers: X-Device-ID: {uuid}

register(platform)     POST /register          → RegisterResponse
fetch_quota()          GET  /quota              → QuotaResponse
report_usage()         POST /usage/report       → QuotaInfo (ou 403 si exceeded)
reset_quota()          POST /quota/reset        → QuotaInfo
```

---

## Arbre d'appels complet

### Depuis un raccourci clavier

```
tauri_plugin_global_shortcut handler (dedicated thread)
└── handle_global_shortcut(app_handle, shortcut_str)
    ├── find_prompt_for_shortcut()
    │   └── normalize_shortcut()
    ├── is_processing = true
    ├── Enigo: release Meta, Shift, Control
    ├── clipboard.write_text("")
    ├── [macOS] osascript → get frontmost app → previous_app
    ├── [macOS] macos_keystroke("c" ou "x")
    │   └── osascript → "keystroke using command down"
    ├── clipboard.read_text() (8 retries × 50ms)
    ├── original_text = clipboard
    ├── last_prompt = prompt
    │
    ├── [show_preview = true]
    │   ├── get/create preview WebviewWindow
    │   ├── window.show() + set_focus()
    │   ├── [macOS] osascript → set frontmost "rewrit"
    │   ├── emit("translation-started")
    │   └── spawn async {
    │       ├── groq.execute_prompt(text, prompt)
    │       ├── pending_translation = result
    │       ├── emit("show-preview", {original, translated})
    │       ├── report_usage_to_backend()
    │       │   └── api_client.report_usage()
    │       │       ├── emit("quota-updated")
    │       │       └── emit("translation-completed")
    │       └── is_processing = false
    │   }
    │
    └── [show_preview = false]
        └── spawn async {
            ├── groq.execute_prompt(text, prompt)
            ├── clipboard.write_text(translated)
            ├── sleep(clipboard_ready)
            ├── spawn_blocking → macos_keystroke("v")
            ├── report_usage_to_backend()
            └── is_processing = false
        }
```

### Depuis un double-tap Ctrl

```
rdev::listen (rdev thread, bloquant)
└── callback(event)
    ├── enabled.load(Relaxed) == false → return
    └── detector.handle_event(event_type) == true
        └── std::thread::spawn
            └── handle_quick_select_trigger(app_handle)
                ├── quick_select_enabled == false → return
                ├── quick_select_open == true → toggle off (hide + refocus) → return
                ├── is_processing = true
                ├── Enigo: release Meta, Shift, Control
                ├── clipboard.write_text("")
                ├── [macOS] osascript → get frontmost app → previous_app
                ├── [macOS] macos_keystroke("c")  (toujours copy, jamais cut)
                ├── clipboard.read_text() (8 retries)
                ├── original_text = clipboard
                ├── get/create quickselect WebviewWindow
                ├── window.show() + set_focus()
                ├── [macOS] osascript → set frontmost "rewrit"
                ├── quick_select_open = true
                ├── emit("quick-select-open")
                └── is_processing = false
                    (la suite depend de l'action utilisateur dans QuickSelect.svelte)
```

### Depuis le menu system tray

```
TrayIconBuilder::on_menu_event
├── "quit" → std::process::exit(0)
└── "settings"
    ├── get_webview_window("main") exists?
    │   ├── Yes → show() + set_focus()
    │   └── No  → WebviewWindowBuilder::new("main", url)
    │             ├── dev:  External(http://localhost:5173)
    │             └── prod: App("index.html")
    │             → 800x600, center, show + focus
    │
    └── (la fenetre charge App.svelte qui monte Settings.svelte)
```

---

## Delais configures (`config.rs`)

Tous les delais sont configurable via variables d'environnement (`.env`).

| Variable | Defaut | Usage |
|----------|--------|-------|
| `DELAY_WINDOW_HIDE` | 150ms | Apres hide preview, avant refocus |
| `DELAY_RESTORE_CLIPBOARD` | 500ms | Avant de restaurer le clipboard original |
| `DELAY_CLIPBOARD_READY` | 100ms | Apres ecriture clipboard, avant Cmd+V |
| `DELAY_KEYBOARD_SHORT_DELAY` | 60ms | Entre press/release modifier et touche |
| `DELAY_AFTER_COPY` | 150ms | Apres Cmd+C/X, avant lecture clipboard |
| `DELAY_CLIPBOARD_RETRY` | 50ms | Entre chaque retry de lecture clipboard |
| `DELAY_AFTER_REFOCUS` | 200ms | Apres refocus app, avant paste |

Charges une seule fois au demarrage via `lazy_static!`.
