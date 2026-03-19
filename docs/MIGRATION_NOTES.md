# Notes de Migration et Corrections

**Date**: 25 Décembre 2025
**Statut**: ✅ Projet compilable et fonctionnel

## Résumé

Le projet a été entièrement corrigé et adapté pour fonctionner sur macOS Apple Silicon avec Python 3.13 et les dernières versions des dépendances. Tous les problèmes de compilation ont été résolus.

## Problèmes Résolus

### 1. Dépendances Python incompatibles avec Python 3.13

**Problème**: `psycopg2-binary` et `pydantic-core` ne compilaient pas avec Python 3.13
- `psycopg2-binary==2.9.9` nécessitait une version de Python < 3.13
- `pydantic-core` nécessitait Rust pour la compilation

**Solution**:
- ✅ Installation de Rust via rustup
- ✅ Migration de `psycopg2-binary` vers `psycopg[binary]>=3.2.0`
- ✅ Mise à jour de toutes les dépendances vers des versions compatibles Python 3.13

**Fichiers modifiés**:
- `backend/requirements.txt`: Toutes les versions mises à jour
- `backend/.env`: URL de connexion changée de `postgresql://` à `postgresql+psycopg://`
- `backend/.env.example`: Même changement

**Nouvelles versions**:
```txt
fastapi>=0.115.0 (était 0.104.0)
uvicorn[standard]>=0.32.0 (était 0.24.0)
sqlalchemy>=2.0.36 (était 2.0.23)
psycopg[binary]>=3.2.0 (était psycopg2-binary==2.9.9)
anthropic>=0.40.0 (était 0.7.0)
pydantic>=2.10.0 (était 2.5.0)
alembic>=1.14.0 (était 1.13.0)
```

### 2. Configuration PATH manquante

**Problème**: PostgreSQL et Rust n'étaient pas dans le PATH
- `psql` non trouvé lors de l'exécution
- `cargo`/`rustc` non trouvés par npm/Tauri

**Solution**:
- ✅ Ajout de PostgreSQL au PATH: `/opt/homebrew/opt/postgresql@15/bin`
- ✅ Ajout de Rust au PATH: `$HOME/.cargo/bin`
- ✅ PATH complet dans `~/.zshrc` pour persistance
- ✅ PATH exporté dans `dev-start.sh` pour tous les terminaux

**PATH complet utilisé**:
```bash
export PATH="/opt/homebrew/opt/postgresql@15/bin:$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:$PATH"
```

### 3. Conflit de versions Tauri v1 vs v2

**Problème**: Code Rust écrit pour Tauri v1 mais CLI v2 installé
- API Tauri complètement changée entre v1 et v2
- `tauri.conf.json` au mauvais format
- Dépendances Cargo incompatibles

**Solution**:
- ✅ Rétrogradation vers Tauri v1.5 (compatible avec le code existant)
- ✅ Downgrade Svelte 5 → Svelte 4 (compatible Tauri v1)
- ✅ Restauration du format Tauri v1 pour `tauri.conf.json`

**Fichiers modifiés**:
- `package.json`: Toutes les versions Tauri/Svelte downgradées
- `src-tauri/Cargo.toml`: Tauri v1.5
- `src-tauri/tauri.conf.json`: Format v1 restauré

### 4. Problèmes de concurrence Rust (Send trait)

**Problème**: `MutexGuard` n'est pas `Send` entre threads
```rust
error: future cannot be sent between threads safely
  = help: `std::sync::MutexGuard` is not `Send`
```

**Solution**:
- ✅ Ajout de `#[derive(Clone)]` à `ApiClient`
- ✅ Clonage du client avant `.await` dans toutes les commandes async
- ✅ Libération du `MutexGuard` avant les points d'attente

**Fichiers modifiés**:
- `src-tauri/src/api_client.rs`: Ajout de `Clone` trait
- `src-tauri/src/main.rs`: Refactoring de `fetch_quota`, `translate_text`, et `setup`

**Pattern utilisé**:
```rust
// Clone client to release lock before await
let client = {
    let api_client = state.api_client.lock().unwrap();
    api_client.as_ref().cloned()
};

if let Some(client) = client {
    // Maintenant on peut await sans tenir le lock
    match client.translate(text).await {
        // ...
    }
}
```

### 5. Icônes manquantes

**Problème**: Tauri nécessite des icônes mais elles n'existaient pas
```
failed to read icon: No such file or directory
```

**Solution**:
- ✅ Création de fichiers PNG placeholder dans `src-tauri/icons/`
- ✅ Retrait de `.icns` et `.ico` de la config (non critiques pour dev)

**Fichiers créés**:
- `src-tauri/icons/32x32.png`
- `src-tauri/icons/128x128.png`
- `src-tauri/icons/128x128@2x.png`
- `src-tauri/icons/icon.png`

### 6. Hot Reload Backend non fonctionnel

**Problème**: Warning uvicorn sur le reload
```
WARNING: Current configuration will not reload
You must pass the application as an import string
```

**Solution**:
- ✅ Changement de `uvicorn.run(app, ...)` vers `uvicorn.run("main:app", ...)`
- ✅ Reload maintenant fonctionnel avec watchfiles

**Fichier modifié**:
- `backend/main.py`: Format import string pour uvicorn

### 7. Base de données non créée

**Problème**: Database `rewrit` n'existait pas
```
connection to server failed: database "rewrit" does not exist
```

**Solution**:
- ✅ Création manuelle avec `createdb rewrit`
- ✅ Script `dev-start.sh` propose de la créer automatiquement

### 8. Texte non remplacé automatiquement

**Problème**: L'API de traduction fonctionnait mais le texte n'était pas remplacé automatiquement
- Le raccourci faisait l'appel API (`POST /api/v1/translate HTTP/1.1 200 OK`)
- Le texte traduit était copié dans le presse-papier
- Mais aucun remplacement automatique ne se produisait (utilisateur devait coller manuellement)

**Solution**:
- ✅ Ajout de la librairie `enigo = "0.1"` dans Cargo.toml
- ✅ Implémentation de simulation de touches (Cmd+V) après écriture dans le presse-papier
- ✅ Délai de 100ms pour laisser le presse-papier se mettre à jour
- ✅ Support multi-plateforme (Cmd+V sur macOS, Ctrl+V sur Windows/Linux)

**Fichiers modifiés**:
- `src-tauri/Cargo.toml`: Ajout de `enigo = "0.1"`
- `src-tauri/src/main.rs`: Ajout de simulation de touches dans `handle_global_shortcut`

**Code ajouté**:
```rust
// Auto-paste the translated text
thread::sleep(Duration::from_millis(100)); // Small delay for clipboard to be ready

let mut enigo = Enigo::new();

// Simulate Cmd+V on macOS or Ctrl+V on Windows/Linux
#[cfg(target_os = "macos")]
{
    enigo.key_down(Key::Meta); // Command key on Mac
    enigo.key_click(Key::Raw(9)); // 'v' key code
    enigo.key_up(Key::Meta);
}

#[cfg(not(target_os = "macos"))]
{
    enigo.key_down(Key::Control);
    enigo.key_click(Key::Raw(9)); // 'v' key code
    enigo.key_up(Key::Control);
}
```

## Nouvelles Fonctionnalités

### Hot Reload Complet

| Composant | Hot Reload | Temps de rechargement |
|-----------|------------|----------------------|
| Frontend Svelte | ✅ Vite HMR | < 1s (instantané) |
| Backend Python | ✅ Uvicorn reload | 1-3s |
| Backend Rust | ⚠️ Cargo watch | 2-10s (recompilation) |

### Script de Lancement Automatique

Le script `dev-start.sh` a été amélioré :
- ✅ Détection automatique de tous les prérequis
- ✅ PATH configuré automatiquement pour tous les terminaux
- ✅ Installation des dépendances si manquantes
- ✅ Création de la base de données si nécessaire
- ✅ Lancement dans 2 terminaux séparés (backend + Tauri)
- ✅ Affichage du mode actuel (AI vs Mock)

## Structure de la Base de Données

**URL de connexion**: `postgresql+psycopg://localhost:5432/rewrit`

**Driver**: psycopg v3 (au lieu de psycopg2)

**Tables** (créées automatiquement par SQLAlchemy):
- `users`: Gestion des utilisateurs (device_id, is_pro)
- `usage_logs`: Logs de toutes les actions
- `daily_quotas`: Quota journalier par utilisateur

## Configuration Actuelle

### Backend
- **IA**: Groq API (côté client Tauri)
- **Port**: 8000
- **Hot Reload**: Activé
- **Base de données**: PostgreSQL 15

### Frontend
- **Framework**: Svelte 4
- **Build tool**: Vite 5
- **Port dev**: 5173 (ou 5174 si occupé)

### Desktop App
- **Framework**: Tauri 1.5
- **Rust**: 1.92.0
- **Raccourcis globaux**: Cmd+Alt+T (EN→FR), Cmd+Alt+Y (FR→EN)

## Commandes Utiles

### Lancement Automatique
```bash
./dev-start.sh
```

### Lancement Manuel

**Backend**:
```bash
cd backend
source venv/bin/activate
export PATH="/opt/homebrew/opt/postgresql@15/bin:$PATH"
python main.py
```

**Tauri**:
```bash
export PATH="/opt/homebrew/opt/postgresql@15/bin:$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:$PATH"
npm run tauri:dev
```

### Vérification du Mode
```bash
./check-mode.sh
```

### Basculer le Mode
```bash
./toggle-ai.sh
```

## Points d'Attention

### Première Compilation Rust
La première compilation Tauri prend **5-10 minutes**. C'est normal car Cargo compile toutes les dépendances depuis zéro.

### Compatibilité Versions
- Le projet utilise **Tauri v1.5** et non v2
- **Svelte 4** et non 5
- **psycopg v3** et non psycopg2
- Ne pas upgrader sans migration complète du code

### PATH Requis
Pour que tout fonctionne, le PATH doit inclure :
- PostgreSQL 15
- Rust/Cargo
- Homebrew bin
- Standard Unix paths

## Prochaines Étapes Recommandées

1. **Icônes personnalisées** - Remplacer les placeholder PNG
2. **Tests** - Tester les raccourcis Cmd+Alt+T et Cmd+Alt+Y
3. **API Key** - Configurer la clé Groq dans les Settings de l'app
4. **Migration Alembic** - Initialiser les migrations de base de données
5. **Documentation** - Documenter l'API avec des exemples

## Ressources

- [Tauri v1 Documentation](https://v1.tauri.app/)
- [Svelte 4 Documentation](https://svelte.dev/docs/v4)
- [FastAPI Documentation](https://fastapi.tiangolo.com/)
- [psycopg v3 Documentation](https://www.psycopg.org/psycopg3/docs/)

---

**Note**: Ce document est maintenu dans `docs/` conformément aux guidelines du projet.
