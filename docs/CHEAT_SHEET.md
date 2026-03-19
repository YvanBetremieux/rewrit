# rewrit - Cheat Sheet 📋

## 🚀 Commandes Essentielles

```bash
# Lancer le projet (tout automatique)
./dev-start.sh
```

## ⌨️ Raccourcis Clavier

| Raccourci | Action |
|-----------|--------|
| `Cmd+Alt+T` | Traduire EN → FR |
| `Cmd+Alt+Y` | Traduire FR → EN |
| Double-tap `Ctrl` | Quick Select (langue + ton) |
| `Cmd+,` | Ouvrir les paramètres |
| `Ctrl+C` | Arrêter tous les processus (terminal) |

## 🌐 URLs Utiles

| URL | Description |
|-----|-------------|
| http://localhost:8000 | API root |
| http://localhost:8000/docs | Documentation Swagger |
| http://localhost:8000/api/v1/status | Statut et configuration |
| http://localhost:8080 | Documentation projet |

## 📂 Fichiers Importants

```
backend/.env              # Configuration principale
src-tauri/src/main.rs     # Entry point, shortcuts, Groq client
src-tauri/src/groq_client.rs  # Groq API client
QUICK_START_FR.md         # Guide rapide en français
```

## 🔍 Vérifications

```bash
# Backend fonctionne?
curl http://localhost:8000

# Statut complet
curl http://localhost:8000/api/v1/status
```

## 🐛 Dépannage Express

| Problème | Solution |
|----------|----------|
| Rust not found | `curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf \| sh` |
| PostgreSQL not found | `brew install postgresql@15` |
| Database not found | `createdb rewrit` |
| Groq API error | Vérifier la clé API Groq dans les settings de l'app |

## 📊 Workflow

```
1. Développement
   └─> ./dev-start.sh
   └─> Code, test, repeat

2. Production
   └─> npm run tauri:build
```

## 🎯 Tests Rapides

```bash
# 1. Lancer
./dev-start.sh

# 2. Ouvrir Notes.app (ou autre)

# 3. Taper "hello"

# 4. Sélectionner + Cmd+Alt+T

# 5. Voir la traduction en français 🎉
```

## 💡 Astuces

```bash
# Voir les logs du backend
# (dans le terminal Backend ouvert par dev-start.sh)

# Rebuild complet de Tauri (si problème)
cd src-tauri && cargo clean && cd .. && npm run tauri:dev
```

## 📱 Structure après lancement

```
Terminal 1 (principal)
  └─> Infos + contrôle
      └─> Ctrl+C ici arrête tout

Terminal 2 (Backend)
  └─> http://localhost:8000
      └─> API FastAPI

Terminal 3 (Frontend)
  └─> App Tauri
      └─> Desktop app
```

## 🎨 Customize

### Changer le port API

```bash
# backend/.env
PORT=8080
```

---

**Besoin d'aide?** → `QUICK_START_FR.md` | `FEATURE_FLAGS.md` | `README.md`
