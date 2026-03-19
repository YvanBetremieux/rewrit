# rewrit - Démarrage Rapide 🚀

## TL;DR - Lancer le projet en 30 secondes

```bash
# 1. Lancer le script automatique
./dev-start.sh

# 2. C'est tout ! 🎉
```

Le script va:
- Vérifier tous les prérequis
- Installer les dépendances manquantes
- Créer la base de données
- Lancer le backend et le frontend dans des terminaux séparés

## Configuration de l'IA

La traduction IA est gérée directement par l'application Tauri via l'API Groq (LLaMA 3.3). La clé API Groq se configure dans les Settings de l'application desktop.

## Structure des terminaux

Après le lancement, vous aurez **3 terminaux**:

1. **Terminal principal** - Celui où vous avez lancé `dev-start.sh`
   - Affiche les informations de lancement
   - Press Ctrl+C ici pour tout arrêter

2. **Terminal Backend** - API Python (FastAPI)
   - http://localhost:8000
   - http://localhost:8000/docs (documentation Swagger)

3. **Terminal Frontend** - Application Tauri
   - L'app desktop s'ouvre automatiquement
   - Icon dans la barre de menu (macOS)

## Tester l'application

1. Ouvrez n'importe quelle app (Notes, TextEdit, etc.)
2. Tapez un texte en anglais
3. Sélectionnez le texte
4. Appuyez sur `Cmd+Alt+T` (traduit EN → FR)
5. Le texte est traduit en français via Groq AI

## Raccourcis clavier

- `Cmd+Alt+T` - Traduire EN → FR
- `Cmd+Alt+Y` - Traduire FR → EN
- Double-tap `Ctrl` - Quick Select (langue + ton)
- `Cmd+,` - Ouvrir les paramètres (dans l'app)
- `Ctrl+C` - Arrêter tous les processus (dans le terminal principal)

## Endpoints utiles

- http://localhost:8000 - Page d'accueil API
- http://localhost:8000/api/v1/status - Statut et configuration
- http://localhost:8000/docs - Documentation interactive

## Arrêter le projet

Dans le terminal principal (celui où vous avez lancé `dev-start.sh`):

```bash
# Appuyez sur Ctrl+C
```

Tous les processus seront arrêtés automatiquement.

## Problèmes courants

### "Rust not found"

```bash
# Installez Rust
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh

# Redémarrez le terminal
# Puis relancez dev-start.sh
```

### "PostgreSQL not found"

```bash
# macOS
brew install postgresql@15
brew services start postgresql@15

# Puis relancez dev-start.sh
```

### "Database rewrit does not exist"

```bash
createdb rewrit
```

## Scripts disponibles

| Script | Description |
|--------|-------------|
| `./dev-start.sh` | Lance tout le projet automatiquement |
| `./quickstart.sh` | Setup complet (première fois) |

## Fichiers importants

- `backend/.env` - Configuration backend (database, port)
- `README.md` - Documentation complète
- `DoD.md` - Spécifications (français)

## Besoin d'aide?

1. Consultez `README.md` pour la doc complète
2. Consultez `IMPLEMENTATION_SUMMARY.md` pour les détails techniques

## Prochaines étapes

Une fois le projet lancé:

1. ✅ Testez les traductions (Cmd+Alt+T / Cmd+Alt+Y)
2. ✅ Ouvrez les settings (icône dans la barre de menu)
3. ✅ Vérifiez le quota (5 actions/jour en mode gratuit)
4. ✅ Consultez l'API: http://localhost:8000/docs

Bon développement! 🎉
