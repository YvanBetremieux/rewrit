# Configuration - rewrit

## Vue d'ensemble

L'architecture de rewrit sépare clairement les responsabilités :
- **Tauri (Rust)** : Gère les raccourcis globaux, le clipboard, et appelle l'API Groq pour la traduction IA
- **Backend (FastAPI)** : Gère uniquement les quotas et l'enregistrement des appareils

## Configuration IA (Groq)

La traduction IA est gérée **côté client** par le Tauri desktop app via l'API Groq.

### Configurer la clé API Groq

1. Obtenez une clé API Groq sur https://console.groq.com/
2. Dans l'application rewrit, ouvrez les Settings (clic droit icône system tray → Settings)
3. Entrez votre clé API Groq dans le champ dédié

Le modèle utilisé est **LLaMA 3.3 70B Versatile**.

## Configuration Backend

### Fichier `backend/.env`

```env
DATABASE_URL=postgresql+psycopg://localhost:5432/rewrit
ENVIRONMENT=development
PORT=8000
```

Le backend ne gère plus la traduction IA. Il sert uniquement pour :
- Enregistrement des appareils (device_id)
- Gestion des quotas (5 actions/jour en mode gratuit)
- Logging des actions

## Endpoint de statut

Vérifiez la configuration via l'API :

```bash
curl http://localhost:8000/api/v1/status
```

## Settings de l'application

Les préférences utilisateur sont accessibles via l'icône system tray → Settings :

- **Groq API Key** : Clé pour les appels IA
- **Show preview before paste** : Afficher une prévisualisation avant le collage
- **Quick Select (double-tap Ctrl)** : Activer/désactiver le picker rapide langue+ton
- **Enable rewrit** : Activer/désactiver l'application

## Bonnes pratiques

1. **Ne committez jamais** les clés API (elles sont stockées côté client)
2. **Quota** : 5 actions gratuites par jour, reset à minuit UTC

---

**Note** : L'ancienne architecture avec `USE_AI` / `ANTHROPIC_API_KEY` dans le backend `.env` a été remplacée par des appels directs à Groq depuis le client Tauri.
