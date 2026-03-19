# 📋 Cahier des Charges - rewrit MVP 1

**Version:** 2.0
**Date:** 25 décembre 2024
**Auteur:** Yvan
**Objectif:** Indépendance financière (10k€/mois MRR)

---

## 1. Vision & Contexte

### 1.1 Problématique

Les utilisateurs perdent du temps à copier-coller du texte dans des LLMs pour effectuer des transformations simples (traduction, reformulation, correction). Cette friction ralentit la productivité.

### 1.2 Solution

**rewrit** : Application desktop Mac/Windows permettant de transformer du texte instantanément via des raccourcis clavier, sans quitter l'application de travail.

### 1.3 Positionnement

> "L'IA au bout de tes doigts" - Assistant d'écriture instantané et frictionless.

---

## 2. Périmètre MVP 1

### 2.1 Objectif MVP 1

Valider le concept avec un prototype fonctionnel minimum permettant de :

- Tester l'UX core (sélection → transformation → remplacement)
- Recueillir les premiers feedbacks utilisateurs
- Valider la stack technique Tauri + Rust + Svelte
- Tester le modèle freemium avec quotas

### 2.2 Timeline

**Durée estimée:** 3-4 semaines

### 2.3 Plateformes cibles

- macOS (10.15+)
- Windows 10/11

### 2.4 Ce qui EST dans le MVP 1 ✅

- ✅ Global shortcut (ex: Cmd+Shift+T)
- ✅ Capture du texte sélectionné
- ✅ 1 action prédéfinie : Traduction EN ↔ FR
- ✅ Remplacement automatique du texte
- ✅ Fenêtre de settings basique
- ✅ Device ID unique généré à l'installation
- ✅ Système de quotas (5 actions/jour gratuit)
- ✅ Affichage du quota restant
- ✅ API backend avec gestion des quotas
- ✅ Indicateur visuel de traitement (loading)

### 2.5 Ce qui N'EST PAS dans le MVP 1 ❌

- ❌ Authentification par email/password
- ❌ Paiement / abonnements
- ❌ Preview avant remplacement
- ❌ Choix d'actions multiples
- ❌ Custom prompts
- ❌ Historique des actions
- ❌ Auto-update
- ❌ Multi-langues (interface)
- ❌ Analytics avancées
- ❌ Synchronisation cross-device


---

## 3. Spécifications Fonctionnelles Détaillées

### 3.1 Workflow utilisateur principal

1. User sélectionne du texte (n'importe quelle app)
2. User presse `Cmd+Shift+T` (Mac) ou `Ctrl+Shift+T` (Windows)
3. rewrit vérifie le quota restant
   - → Si quota épuisé : affiche message + lien upgrade
   - → Si quota OK : continue
4. rewrit capture le texte
5. rewrit affiche un indicateur de traitement
6. rewrit appelle l'API backend avec `device_id`
7. Backend vérifie quota côté serveur
8. Backend détecte la langue (EN ou FR)
9. Backend traduit vers l'autre langue via LLM
10. Backend décrémente le quota utilisateur
11. rewrit reçoit la réponse + quota mis à jour
12. rewrit remplace le texte sélectionné
13. rewrit met à jour l'affichage du quota
14. Indicateur disparaît

**Durée totale cible:** <3 secondes
### 3.2 Système d'identification utilisateur

**Device ID:**

- Généré automatiquement au premier lancement
- UUID v4 stocké localement
- Envoyé avec chaque requête API
- Anonyme : pas de données personnelles

**Stockage:**
- `~/.rewrit/device_id`
- Format: `"550e8400-e29b-41d4-a716-446655440000"`

**Propriétés:**

- Unique par installation
- Persistant (ne change jamais)
- Si fichier supprimé → nouveau device_id → nouveau quota

### 3.3 Système de quotas

**Règles MVP 1:**

- 5 actions gratuites par jour (reset à minuit UTC)
- Quota partagé pour toutes les actions
- Compteur côté serveur (source de vérité)
- Compteur local (cache, peut être désynchronisé)

**Comportement quota épuisé:**

```
┌─────────────────────────────────────┐
│  ⚠️  Daily Limit Reached            │
│                                     │
│  You've used your 5 free actions   │
│  today. Your quota resets in:      │
│                                     │
│        🕐  3h 24min                 │
│                                     │
│  Want unlimited actions?           │
│  [Upgrade to Pro] (coming soon)    │
│                                     │
│         [Close]                     │
└─────────────────────────────────────┘
```
### 3.4 Détection de langue

**Règles de détection:**

- Si >50% caractères latins + mots anglais courants → EN
- Si accents français (é, è, à) ou mots français → FR
- Par défaut si incertain → EN vers FR

### 3.5 Fenêtre Settings

**Contenu minimal:**

- ✅ Affichage quota restant : "3/5 actions today"
- ✅ Heure de reset quota : "Resets at midnight UTC"
- ✅ Toggle Enable/Disable rewrit
- ✅ Affichage du raccourci clavier (non modifiable en MVP1)
- ✅ Device ID (lecture seule, pour support)
- ✅ Lien vers documentation
- ✅ Lien "Upgrade to Pro" (placeholder pour MVP1)

**Accès:**

- Menu bar icon (clic droit) → Settings
- Ou raccourci `Cmd+,` (Mac convention)

**Wireframe Settings mis à jour:**

```
┌─────────────────────────────────────────┐
│  rewrit - Settings                    ⓧ │
├─────────────────────────────────────────┤
│                                         │
│  📊 Usage                               │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │   3 / 5 actions used today      │   │
│  │   ▓▓▓▓▓▓░░░░░░░░░░░░  60%       │   │
│  └─────────────────────────────────┘   │
│                                         │
│  ⏰ Quota resets in 4h 23min            │
│                                         │
│  [✨ Upgrade to Pro] (coming soon)      │
│                                         │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   │
│                                         │
│  🔧 Preferences                         │
│                                         │
│  ☑️ Enable rewrit                       │
│  ☑️ Show notifications                  │
│                                         │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   │
│                                         │
│  ⌨️  Shortcuts                          │
│                                         │
│  Translate: Cmd+Shift+T                 │
│                                         │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   │
│                                         │
│  🆔 Device ID (for support)             │
│  550e8400-e29b-41d4-a716-446655440000   │
│                                         │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   │
│                                         │
│  📚 Documentation  •  ❓ Support         │
│                                         │
│              [Close]                    │
│                                         │
└─────────────────────────────────────────┘
```
### 3.6 Indicateur visuel

**Toast normal (action en cours):**

```
┌─────────────────────────────┐
│  🔄 rewrit is working...    │
│     2/5 actions used        │
└─────────────────────────────┘
```

**Toast quota warning (dernière action):**

```
┌─────────────────────────────┐
│  ✅ Translation complete     │
│  ⚠️  5/5 actions used today  │
│     Resets in 6h 15min      │
└─────────────────────────────┘
```

**Position:** Top-right, 20px margin

### 3.7 Gestion des erreurs

**Cas d'erreur à gérer:**

| Erreur | Message utilisateur | Action |
|--------|---------------------|--------|
| Aucun texte sélectionné | "Please select text first" | Toast 2s |
| Quota épuisé | "Daily limit reached (5/5)" + modal | Ouvre modal upgrade |
| Erreur API (500) | "Service temporarily unavailable" | Toast 3s |
| Timeout (>10s) | "Request timed out, try again" | Toast 3s |
| Texte trop long (>5000 char) | "Text too long (max 5000 chars)" | Toast 3s |
| Pas de connexion internet | "No internet connection" | Toast 3s |
| Device ID invalide | "Please reinstall rewrit" | Toast 5s |

---

## 4. Spécifications Techniques

### 4.1 Stack Technologique

**Frontend (Interface):**

- Framework : Svelte 4
- Build : Vite
- Styling : TailwindCSS
- Icons : Lucide Svelte

**Backend App (Core Logic):**

- Framework : Tauri 1.5+
- Langage : Rust 1.70+
- Async runtime : tokio
- HTTP client : reqwest
- UUID generation : uuid crate

**Backend API:**

- Framework : FastAPI (Python 3.11+)
- Database : PostgreSQL (Railway managed)
- ORM : SQLAlchemy
- LLM : Claude Haiku 4.5 API
- Déploiement : Railway.app

### 4.2 Architecture Système

```
┌─────────────────────────────────────────┐
│   User's App (Word, Notes, Browser...) │
│   ┌─────────────────────────────────┐   │
│   │ Selected Text: "Hello world"    │   │
│   └─────────────────┬───────────────┘   │
└─────────────────────┼───────────────────┘
                      │
                      │ Cmd+Shift+T
                      ▼
┌─────────────────────────────────────────┐
│         rewrit (Tauri App)              │
│  ┌──────────────────────────────────┐   │
│  │  Rust Backend                    │   │
│  │  - Global hotkey listener        │   │
│  │  - Clipboard manager             │   │
│  │  - Text replacement engine       │   │
│  │  - Device ID manager             │   │
│  │  - Local quota cache             │   │
│  └──────────────┬───────────────────┘   │
│                 │                        │
│  ┌──────────────▼───────────────────┐   │
│  │  Svelte Frontend                 │   │
│  │  - Settings UI                   │   │
│  │  - Toast notifications           │   │
│  │  - Quota display                 │   │
│  └──────────────────────────────────┘   │
└─────────────────┬───────────────────────┘
                  │
                  │ HTTPS POST
                  │ + device_id header
                  ▼
┌─────────────────────────────────────────┐
│      Backend API (FastAPI)              │
│  POST /api/v1/translate                 │
│  ┌──────────────────────────────────┐   │
│  │ 1. Verify device_id              │   │
│  │ 2. Check quota (PostgreSQL)      │   │
│  │ 3. If OK: Call Claude API        │   │
│  │ 4. Decrement quota               │   │
│  │ 5. Return result + quota         │   │
│  └──────────────────────────────────┘   │
│                                         │
│  ┌──────────────────────────────────┐   │
│  │  PostgreSQL Database             │   │
│  │  - users table (device_id)       │   │
│  │  - usage_logs table              │   │
│  │  - quota_tracking                │   │
│  └──────────────────────────────────┘   │
└─────────────────────────────────────────┘
```
### 4.3 Schéma Base de Données

**Table : users**

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    device_id UUID UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    last_seen_at TIMESTAMP DEFAULT NOW(),
    total_actions_count INTEGER DEFAULT 0,
    is_pro BOOLEAN DEFAULT FALSE
);

CREATE INDEX idx_device_id ON users(device_id);
```

**Table : usage_logs**

```sql
CREATE TABLE usage_logs (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    action_type VARCHAR(50) NOT NULL, -- 'translate'
    input_text_length INTEGER,
    output_text_length INTEGER,
    processing_time_ms INTEGER,
    success BOOLEAN,
    error_message TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_user_created ON usage_logs(user_id, created_at);
```

**Table : daily_quotas**

```sql
CREATE TABLE daily_quotas (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    quota_date DATE NOT NULL,
    actions_used INTEGER DEFAULT 0,
    quota_limit INTEGER DEFAULT 5,
    UNIQUE(user_id, quota_date)
);

CREATE INDEX idx_user_date ON daily_quotas(user_id, quota_date);
```

### 4.4 API Backend Endpoints

#### POST /api/v1/translate

**Headers:**
```
X-Device-ID: 550e8400-e29b-41d4-a716-446655440000
Content-Type: application/json
```

**Request:**
```json
{
  "text": "Hello world, how are you?"
}
```

**Response (Success):**
```json
{
  "success": true,
  "original_text": "Hello world, how are you?",
  "translated_text": "Bonjour le monde, comment allez-vous ?",
  "detected_language": "en",
  "target_language": "fr",
  "processing_time_ms": 1234,
  "quota": {
    "used": 3,
    "limit": 5,
    "remaining": 2,
    "resets_at": "2024-12-26T00:00:00Z"
  }
}
```

**Response (Quota Exceeded):**
```json
{
  "success": false,
  "error": "quota_exceeded",
  "message": "Daily limit of 5 actions reached",
  "quota": {
    "used": 5,
    "limit": 5,
    "remaining": 0,
    "resets_at": "2024-12-26T00:00:00Z"
  }
}
```

**Response (Error):**
```json
{
  "success": false,
  "error": "invalid_device_id",
  "message": "Device ID is invalid or missing"
}
```

---

#### GET /api/v1/quota

**Headers:**
```
X-Device-ID: 550e8400-e29b-41d4-a716-446655440000
```

**Response:**
```json
{
  "quota": {
    "used": 3,
    "limit": 5,
    "remaining": 2,
    "resets_at": "2024-12-26T00:00:00Z"
  },
  "is_pro": false
}
```

---

#### POST /api/v1/register

Appelé automatiquement au premier lancement de l'app.

**Headers:**
```
X-Device-ID: 550e8400-e29b-41d4-a716-446655440000
Content-Type: application/json
```

**Request:**
```json
{
  "platform": "macos",
  "app_version": "1.0.0"
}
```

**Response:**
```json
{
  "success": true,
  "device_id": "550e8400-e29b-41d4-a716-446655440000",
  "quota": {
    "used": 0,
    "limit": 5,
    "remaining": 5,
    "resets_at": "2024-12-26T00:00:00Z"
  }
}
```
### 4.5 Structure de données locale

**Config file : `~/.rewrit/config.json`**

```json
{
  "version": "1.0.0",
  "device_id": "550e8400-e29b-41d4-a716-446655440000",
  "enabled": true,
  "shortcuts": {
    "translate": "CommandOrControl+Shift+T"
  },
  "preferences": {
    "show_notifications": true,
    "sound_enabled": false
  },
  "quota_cache": {
    "used": 3,
    "limit": 5,
    "last_sync": "2024-12-25T14:30:00Z"
  }
}
```

**Device ID file : `~/.rewrit/device_id`**

```
550e8400-e29b-41d4-a716-446655440000
```

### 4.6 Sécurité

**API Key (côté serveur uniquement):**

- Stockée dans variable d'environnement backend
- Jamais exposée au client
- Rotation possible sans impacter les utilisateurs

**Device ID:**

- UUID v4 généré côté client
- Validation côté serveur
- Pas de données personnelles liées

**Rate Limiting:**

- 1 requête/seconde par device_id
- Protection DDoS : max 100 requêtes/heure par IP
- Quota journalier : 5 actions/device_id

**Anti-abus:**

- Log de toutes les actions
- Détection devices suspects (trop de requêtes échouées)
- Possibilité de blacklist device_id si abuse


---

## 5. User Stories & Critères d'Acceptance

### US-001 : Installer l'application

**En tant qu'** utilisateur
**Je veux** télécharger et installer rewrit facilement
**Afin de** commencer à l'utiliser rapidement

**Critères d'acceptance:**

- ✅ DMG disponible pour macOS (non signé, warning OK)
- ✅ EXE/MSI disponible pour Windows (non signé, SmartScreen OK)
- ✅ Installation en <2 minutes
- ✅ rewrit démarre automatiquement après installation
- ✅ Device ID généré automatiquement au premier lancement
- ✅ Appel API /register réussi
- ✅ Icon dans la menu bar (Mac) / system tray (Windows)


### US-002 : Voir mon quota

**En tant qu'** utilisateur gratuit
**Je veux** voir combien d'actions il me reste
**Afin de** savoir quand je vais atteindre la limite

**Critères d'acceptance:**

- ✅ Settings affiche "X/5 actions used today"
- ✅ Progress bar visuelle du quota
- ✅ Affichage de l'heure de reset "Resets in Xh Ymin"
- ✅ Quota mis à jour après chaque action
- ✅ Synchronisation avec le serveur au lancement

### US-003 : Traduire du texte EN→FR

**En tant qu'** utilisateur
**Je veux** traduire du texte anglais vers le français
**Afin de** communiquer avec des francophones

**Critères d'acceptance:**

- ✅ Je sélectionne "Hello, how are you?" dans n'importe quelle app
- ✅ Je presse `Cmd+Shift+T` (Mac) ou `Ctrl+Shift+T` (Win)
- ✅ Toast "rewrit is working... 2/5 actions used" apparaît
- ✅ Texte remplacé par "Bonjour, comment allez-vous ?" en <3s
- ✅ Toast disparaît avec fade out
- ✅ Quota local et serveur décrémenté
- ✅ Curseur positionné après le texte remplacé

### US-004 : Atteindre la limite quotidienne

**En tant qu'** utilisateur gratuit
**Je veux** être prévenu quand j'atteins ma limite
**Afin de** comprendre pourquoi ça ne marche plus

**Critères d'acceptance:**

- ✅ Après ma 5ème action, toast warning "5/5 actions used"
- ✅ 6ème tentative déclenche modal "Daily Limit Reached"
- ✅ Modal affiche le temps avant reset
- ✅ Bouton "Upgrade to Pro (coming soon)" présent
- ✅ Modal se ferme avec [X] ou [Close]
- ✅ Aucune action n'est effectuée

### US-005 : Reset quotidien automatique

**En tant qu'** utilisateur
**Je veux** récupérer mes 5 actions chaque jour
**Afin de** continuer à utiliser rewrit gratuitement

**Critères d'acceptance:**

- ✅ Quota reset à minuit UTC
- ✅ Au lancement après reset, quota affiché = 0/5
- ✅ Première action du jour fonctionne
- ✅ Compteur incrémente correctement

### US-006 : Gérer les erreurs gracieusement

**En tant qu'** utilisateur
**Je veux** comprendre ce qui ne va pas quand ça échoue
**Afin de** corriger le problème

**Critères d'acceptance:**

- ✅ Si aucun texte sélectionné → Toast "Please select text first"
- ✅ Si quota épuisé → Modal "Daily Limit Reached"
- ✅ Si erreur réseau → Toast "No internet connection"
- ✅ Si timeout → Toast "Request timed out"
- ✅ Aucun crash de l'app, toujours récupérable
- ✅ Les erreurs ne décrémentent pas le quota

### US-007 : Support technique

**En tant qu'** utilisateur ayant un problème
**Je veux** partager mon Device ID avec le support
**Afin d'** obtenir de l'aide

**Critères d'acceptance:**

- ✅ Device ID visible dans Settings
- ✅ Bouton "Copy" à côté du Device ID
- ✅ Copie dans le clipboard réussie
- ✅ Toast confirmation "Device ID copied"


---

## 6. Logique Backend

### 6.1 Algorithme de vérification quota

```python
def check_and_decrement_quota(device_id: str) -> QuotaResult:
    # 1. Get or create user
    user = get_or_create_user(device_id)

    # 2. Get today's quota
    today = date.today()
    quota = get_or_create_daily_quota(user.id, today)

    # 3. Check if quota exceeded
    if quota.actions_used >= quota.quota_limit:
        return QuotaResult(
            allowed=False,
            used=quota.actions_used,
            limit=quota.quota_limit,
            resets_at=get_next_midnight_utc()
        )

    # 4. Increment usage
    quota.actions_used += 1
    db.commit()

    # 5. Log the action
    log_usage(user.id, action_type="translate")

    return QuotaResult(
        allowed=True,
        used=quota.actions_used,
        limit=quota.quota_limit,
        resets_at=get_next_midnight_utc()
    )
```

### 6.2 Tâche CRON de reset quotidien

```python
# Runs daily at 00:00 UTC
@scheduler.scheduled_job('cron', hour=0, minute=0)
def reset_daily_quotas():
    yesterday = date.today() - timedelta(days=1)

    # Archive old quotas (optional)
    archive_old_quotas(yesterday)

    # New quotas will be created on-demand for today
    logger.info("Daily quotas reset completed")
```

---

## 7. Gestion de Projet

### 7.1 Phases de développement

**Semaine 1 : Setup & Core**

- Jour 1-2 : Setup projet Tauri + Svelte
- Jour 3-4 : Global hotkey + clipboard capture
- Jour 5-6 : Text replacement (tests manuels)
- Jour 7 : Device ID generation & storage

**Semaine 2 : Backend API**

- Jour 1-2 : Setup FastAPI + PostgreSQL
- Jour 2-3 : Endpoints /register, /quota, /translate
- Jour 4-5 : Quota logic + tests
- Jour 6 : Claude API integration
- Jour 7 : Error handling

**Semaine 3 : Integration & UI**

- Jour 1-2 : Frontend ↔ Backend communication
- Jour 3-4 : Settings UI avec quota display
- Jour 5 : Toast notifications & modals
- Jour 6 : Edge cases & error handling
- Jour 7 : Polish UI

**Semaine 4 : Testing & Packaging**

- Jour 1-3 : Tests manuels extensifs (Mac + Windows)
- Jour 4-5 : Bug fixes
- Jour 6 : Build & packaging (.dmg + .exe)
- Jour 7 : Documentation + beta distribution

### 7.2 Critères de succès MVP 1

**Critères techniques:**

- ✅ L'app démarre sans erreur sur Mac & Windows
- ✅ Le raccourci fonctionne dans 5+ apps
- ✅ Latence moyenne <3s pour traduction
- ✅ 0 crash pendant 1h d'utilisation intensive
- ✅ Taux de réussite traduction >95%
- ✅ Quota tracking 100% fiable
- ✅ Pas de bypass possible du quota

**Critères produit:**

- ✅ 10 beta testeurs ont réussi à installer et utiliser
- ✅ Feedback positif sur l'UX core (simple & rapide)
- ✅ Compréhension claire du modèle freemium
- ✅ Aucun bloquant majeur remonté
- ✅ Décision GO/NO-GO pour Phase 2


---

## 8. Dépendances & Outils

### 8.1 Développement

**Rust crates nécessaires:**

```toml
[dependencies]
tauri = "1.5"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
global-hotkey = "0.4"
clipboard = "0.5"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = "0.4"
```

**NPM packages:**

```json
{
  "svelte": "^4.0.0",
  "vite": "^5.0.0",
  "tailwindcss": "^3.3.0",
  "lucide-svelte": "^0.300.0"
}
```

**Python (Backend API):**

```txt
fastapi==0.104.0
uvicorn[standard]==0.24.0
sqlalchemy==2.0.23
psycopg2-binary==2.9.9
anthropic==0.7.0
python-dotenv==1.0.0
pydantic==2.5.0
```

### 8.2 Services externes

- **LLM API:** Anthropic Claude API (claude-haiku-4-5-20251001)
- **Database:** PostgreSQL (Railway managed)
- **Hosting API:** Railway.app
- **Analytics:** Aucun pour MVP1


---

## 9. Coûts & Économie

### 9.1 Coûts estimés MVP 1

**Infrastructure mensuelle:**

- Railway PostgreSQL : 5€/mois
- Railway API hosting : 5€/mois (tier Hobby)
- **Total infra : 10€/mois**

**API Claude:**

- Coût par action : ~0,002€
- 100 users × 5 actions/jour × 30 jours = 15 000 actions/mois
- **Coût API : 30€/mois**

**TOTAL MVP 1 : ~40€/mois**

### 9.2 Break-even

Pour couvrir 40€/mois avec modèle payant (9,99€) :

- Besoin de **4 paying users**
- Objectif réaliste à M3-M6


---

## 10. Risques & Mitigation

| Risque | Impact | Probabilité | Mitigation |
|--------|--------|-------------|------------|
| Users contournent quota (réinstall) | 🟡 Moyen | Élevé | Accept pour MVP1, fingerprinting en Phase 2 |
| Coût API explose (abuse) | 🔴 Élevé | Moyen | Rate limiting strict, monitoring quotidien |
| Text replacement ne marche pas | 🔴 Élevé | Moyen | Tester top 10 apps, fallback clipboard |
| DB down = app inutilisable | 🔴 Élevé | Faible | Fallback mode offline (cache local) |
| Permissions macOS bloquent | 🔴 Élevé | Moyen | Doc claire Accessibility |

---

## 11. Métriques de Succès

### Pour le MVP 1

**Métriques techniques:**

- Uptime API : >99%
- Latence moyenne : <3s
- Taux d'erreur : <5%
- Quota bypass rate : <10%

**Métriques produit:**

- 20+ beta testeurs
- 50+ actions/jour (total)
- Taux de satisfaction >8/10
- 10% d'users atteignent limite quotidienne
- Intention d'achat version Pro : >30%

### KPIs à tracker

**Database analytics:**

- Nouveaux users/jour
- Actions totales/jour
- % users atteignant quota
- Distribution actions par user
- Retention Day 1, Day 7

**Support:**

- Nombre de tickets
- Type d'erreurs les plus fréquentes


---

## 12. Next Steps (Post-MVP 1)

Si MVP 1 validé → **Phase 2 (semaines 5-8):**

- Multi-actions (5 prédéfinies : translate, rephrase, fix, summarize, expand)
- Preview avant remplacement
- Custom prompts (1 slot)
- Stripe integration + subscription Pro
- Dashboard web pour gérer compte
- Email notifications (quota warnings)


---

## 13. Annexes

### 13.1 Variables d'environnement Backend

```env
# .env
DATABASE_URL=postgresql://user:pass@host:5432/rewrit
ANTHROPIC_API_KEY=sk-ant-xxxxx
ENVIRONMENT=production
PORT=8000
ALLOWED_ORIGINS=*
RATE_LIMIT_PER_SECOND=1
RATE_LIMIT_PER_HOUR=100
```

### 13.2 Commandes utiles

**Setup DB:**

```bash
# Create tables
python -m alembic upgrade head

# Seed data (optional)
python scripts/seed_db.py
```

**Run backend locally:**

```bash
uvicorn main:app --reload --port 8000
```

**Build app:**

```bash
# macOS
npm run tauri build -- --target universal-apple-darwin

# Windows
npm run tauri build -- --target x86_64-pc-windows-msvc
```

---

**Document vivant - Dernière mise à jour : 25/12/2024 v2.0**

### Changements majeurs :

- ✅ Plus de champ API key côté client
- ✅ Device ID anonyme pour identification
- ✅ Système de quotas complet (5/jour gratuit)
- ✅ Backend avec PostgreSQL pour tracking
- ✅ UI Settings avec affichage quota
- ✅ Prêt pour monetization future

Le scope est un peu plus large (3-4 semaines au lieu de 2-3), mais c'est la base solide pour le modèle freemium.