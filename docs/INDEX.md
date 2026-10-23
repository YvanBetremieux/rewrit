# rewrit - Index de la Documentation 📚

Bienvenue! Ce fichier vous guide vers la bonne documentation selon vos besoins.

---

## 🚀 Je veux juste démarrer le projet

**→ [`QUICK_START_FR.md`](QUICK_START_FR.md)**

Guide de démarrage en 30 secondes. Parfait pour commencer sans lire 50 pages.

**Commande:**
```bash
./dev-start.sh
```

---

## 📋 Je cherche une commande rapide

**→ [`CHEAT_SHEET.md`](CHEAT_SHEET.md)**

Toutes les commandes essentielles, raccourcis clavier, URLs utiles, en format rapide.

---

## 🎯 Je veux comprendre la configuration

**→ [`FEATURE_FLAGS.md`](FEATURE_FLAGS.md)**

Configuration de l'IA (Groq), du backend, et des settings de l'application.

**Résumé:**
- L'IA (Groq/LLaMA) est gérée côté client (Tauri)
- Le backend gère les quotas et l'enregistrement

---

## 🆕 Quoi de neuf?

**→ [`NOUVELLES_FONCTIONNALITES.md`](NOUVELLES_FONCTIONNALITES.md)**

Liste complète de tous les nouveaux fichiers, scripts, et fonctionnalités ajoutées.

**Highlights:**
- Script de lancement automatique (`dev-start.sh`)
- Scripts utilitaires

---

## 🔧 Notes de Migration

**→ [`MIGRATION_NOTES.md`](MIGRATION_NOTES.md)**

Détails complets de tous les problèmes résolus et changements effectués pour rendre le projet fonctionnel.

**Contient:**
- Tous les problèmes résolus (Python 3.13, Tauri, Rust, etc.)
- Solutions appliquées
- Configuration PATH
- Hot reload setup
- Commandes utiles

---

## 📖 Documentation complète

**→ [`README.md`](README.md)**

Documentation principale et complète du projet.

**Sections:**
- Vue d'ensemble
- Installation
- Configuration
- Développement
- Build production
- Architecture
- Troubleshooting

---

## 📝 Spécifications officielles

**→ [`DoD.md`](DoD.md)**

Cahier des charges complet (en français), MVP 1.

**Contient:**
- Vision et objectifs
- Spécifications fonctionnelles
- Spécifications techniques
- User stories
- Base de données
- Timeline

---

## 🏗️ Détails d'implémentation

**→ [`IMPLEMENTATION_SUMMARY.md`](IMPLEMENTATION_SUMMARY.md)**

Résumé de l'implémentation, structure du projet, checklist de tests.

**Contient:**
- État du projet
- Structure complète
- Features implémentées
- Étapes de setup
- Checklist de tests

---

## 🧠 Contexte pour AI

**→ [`CLAUDE.MD`](CLAUDE.MD)**

Fichier de contexte pour l'IA assistant, explique le projet, conventions, architecture.

---

## 🛠️ Setup initial

**→ [`SETUP.md`](SETUP.md)**

Guide de setup des prérequis (Rust, Node, Python, PostgreSQL).

---

## 📁 Structure du Projet

```
rewrit/
├── 📚 DOCUMENTATION
│   ├── INDEX.md                    ← Vous êtes ici
│   ├── QUICK_START_FR.md           ← Démarrage rapide
│   ├── CHEAT_SHEET.md              ← Aide-mémoire
│   ├── FEATURE_FLAGS.md            ← Configuration
│   ├── NOUVELLES_FONCTIONNALITES.md ← Nouveautés
│   ├── MIGRATION_NOTES.md          ← Problèmes résolus ⭐
│   ├── README.md                   ← Doc principale
│   ├── DoD.md                      ← Spécifications
│   ├── IMPLEMENTATION_SUMMARY.md   ← Implémentation
│   ├── SETUP.md                    ← Setup prérequis
│   └── CLAUDE.MD                   ← Contexte AI
│
├── 🔧 SCRIPTS
│   ├── dev-start.sh                ← Lancement auto ⭐
│   └── quickstart.sh               ← Setup initial
│
├── 💻 CODE
│   ├── src/                        ← Frontend Svelte
│   ├── src-tauri/                  ← Backend Rust
│   └── backend/                    ← API FastAPI
│       ├── config.py               ← Feature flags ⭐
│       ├── main.py                 ← API endpoints
│       ├── models.py               ← Database models
│       └── .env                    ← Configuration
│
└── 🎨 ASSETS
    └── public/
```

---

## 🎯 Guides par Cas d'Usage

### Je débute sur le projet

1. [`QUICK_START_FR.md`](QUICK_START_FR.md) - Démarrer
2. [`CHEAT_SHEET.md`](CHEAT_SHEET.md) - Commandes utiles
3. [`README.md`](README.md) - Approfondir

### Je veux développer

1. [`QUICK_START_FR.md`](QUICK_START_FR.md) - Lancer le projet
2. [`FEATURE_FLAGS.md`](FEATURE_FLAGS.md) - Comprendre la configuration

### Je veux configurer l'IA

1. Console Groq: https://console.groq.com/
2. [`FEATURE_FLAGS.md`](FEATURE_FLAGS.md) - Configuration
3. Entrer la clé API Groq dans les Settings de l'app

### Je cherche une info technique

1. [`README.md`](README.md) - Architecture, API
2. [`DoD.md`](DoD.md) - Spécifications détaillées
3. [`IMPLEMENTATION_SUMMARY.md`](IMPLEMENTATION_SUMMARY.md) - Code

### Je veux contribuer

1. [`CLAUDE.MD`](CLAUDE.MD) - Conventions et contexte
2. [`README.md`](README.md) - Architecture
3. [`DoD.md`](DoD.md) - Spécifications

### J'ai un problème

1. [`UPDATES_AND_DEBUG.md`](UPDATES_AND_DEBUG.md) - Logs, diagnostics, mises à jour et releases
2. [`CHEAT_SHEET.md`](CHEAT_SHEET.md) - Dépannage express
3. [`README.md`](README.md) - Section Troubleshooting

---

## 📊 Documentation par Type

### 🚀 Quick Start
- [`QUICK_START_FR.md`](QUICK_START_FR.md) - Guide rapide
- [`CHEAT_SHEET.md`](CHEAT_SHEET.md) - Aide-mémoire

### 📖 Complète
- [`README.md`](README.md) - Documentation principale
- [`DoD.md`](DoD.md) - Spécifications officielles

### 🔧 Technique
- [`ARCHITECTURE.md`](ARCHITECTURE.md) - Architecture technique detaillee (threads, flows, diagrammes)
- [`IMPLEMENTATION_SUMMARY.md`](IMPLEMENTATION_SUMMARY.md) - Implémentation
- [`FEATURE_FLAGS.md`](FEATURE_FLAGS.md) - Configuration
- [`CLAUDE.MD`](CLAUDE.MD) - Contexte technique

### 🆕 Nouveautés
- [`NOUVELLES_FONCTIONNALITES.md`](NOUVELLES_FONCTIONNALITES.md) - Ajouts récents

### ⚙️ Setup
- [`SETUP.md`](SETUP.md) - Prérequis

---

## 🔍 Recherche Rapide

### Commandes
→ [`CHEAT_SHEET.md`](CHEAT_SHEET.md)

### Configuration
→ [`FEATURE_FLAGS.md`](FEATURE_FLAGS.md)

### API Endpoints
→ [`README.md`](README.md) ou [`DoD.md`](DoD.md)

### Base de données
→ [`DoD.md`](DoD.md) - Section 4.3

### Architecture
→ [`README.md`](README.md) ou [`CLAUDE.MD`](CLAUDE.MD)

### Scripts
→ `./` (racine du projet)
- `dev-start.sh` - Lancement

---

## 💡 Recommandations

### Première fois
1. [`QUICK_START_FR.md`](QUICK_START_FR.md)
2. Lancer: `./dev-start.sh`
3. Tester une traduction
4. Lire: [`CHEAT_SHEET.md`](CHEAT_SHEET.md)

### Développement quotidien
1. `./dev-start.sh` (lancer)
2. [`CHEAT_SHEET.md`](CHEAT_SHEET.md) (référence)

### Avant de merger
1. Tester avec l'API Groq
2. Relire [`DoD.md`](DoD.md) (vérifier specs)

### Problème?
1. [`CHEAT_SHEET.md`](CHEAT_SHEET.md) - Dépannage
2. [`README.md`](README.md) - Troubleshooting

---

## 📞 Besoin d'Aide?

1. **Quick answer** → [`CHEAT_SHEET.md`](CHEAT_SHEET.md)
2. **Guide complet** → [`README.md`](README.md)
3. **Feature flags** → [`FEATURE_FLAGS.md`](FEATURE_FLAGS.md)
4. **Nouveautés** → [`NOUVELLES_FONCTIONNALITES.md`](NOUVELLES_FONCTIONNALITES.md)

---

**Bonne lecture et bon code! 🚀**
