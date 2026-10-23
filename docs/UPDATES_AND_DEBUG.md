# Mises à jour et débogage

## Mises à jour automatiques

rewrit utilise `tauri-plugin-updater`, l'équivalent Tauri de Sparkle (Sparkle ne fonctionne qu'avec les apps natives Swift/AppKit).

- Au lancement, puis toutes les 24 h, l'app lit `https://github.com/YvanBetremieux/rewrit/releases/latest/download/latest.json`.
- Si une version plus récente existe :
  - **Install automatically** activé (par défaut) : téléchargement et installation en tâche de fond, puis une boîte de dialogue propose de redémarrer.
  - désactivé : une boîte de dialogue propose d'installer et de redémarrer.
- Réglages › **Updates** : version installée, date de la dernière vérification, bouton **Check for updates…**, bouton **Install & restart**, interrupteurs **Check automatically** et **Install automatically**.
- Les réglages sont stockés dans `~/.rewrit/update_auto_check`, `update_auto_install` et `update_last_check`.

Les versions antérieures à 1.3.0 n'embarquent pas ce mécanisme : il faut installer la première version qui le contient à la main (DMG de la release).

### Publier une release

1. Monter la version dans `package.json`, `src-tauri/Cargo.toml` et `src-tauri/tauri.conf.json`.
2. Commit sur `main`, puis `git tag vX.Y.Z && git push origin main vX.Y.Z`.
3. La CI (`.github/workflows/release.yml`) construit macOS arm64 + x64, Windows et Linux, signe les archives de mise à jour, **publie** la release (plus de brouillon) et y joint `latest.json`.

Le repo doit rester **public** : l'app télécharge `latest.json` et les binaires sans authentification.

### Clé de signature

- Clé privée : `~/.tauri/rewrit.key` (jamais commitée).
- Mot de passe : trousseau macOS, `security find-generic-password -a rewrit -s rewrit-tauri-signing-password -w`.
- Secrets GitHub : `TAURI_SIGNING_PRIVATE_KEY` et `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
- Clé publique : `plugins.updater.pubkey` dans `src-tauri/tauri.conf.json`.

Ne pas générer de clé sans mot de passe : le CLI Tauri ne sait pas la relire (« Wrong password for that key »). Changer de clé rend les versions déjà installées incapables de valider les mises à jour suivantes.

Pour vérifier la signature en local :

```bash
export TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/rewrit.key)"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="$(security find-generic-password -a rewrit -s rewrit-tauri-signing-password -w)"
npm run tauri build -- --bundles app   # doit produire rewrit.app.tar.gz.sig
```

## Débogage

### Réglages › Diagnostics

- **Test AI connection** : envoie un petit prompt au fournisseur actif et affiche la réponse ou l'erreur brute de l'API (clé invalide, modèle retiré, quota…).
- **Open logs** : ouvre le dossier des logs.
- **Copy diagnostics** : copie version, OS, fournisseur, modèle, dossier de logs, dernière vérification de mise à jour et résultat du test.

### Fichier de logs

| OS | Emplacement |
|----|-------------|
| macOS | `~/Library/Logs/com.rewrit.app/rewrit.log` |
| Windows | `%LOCALAPPDATA%\com.rewrit.app\logs\rewrit.log` |
| Linux | `~/.local/share/com.rewrit.app/logs/rewrit.log` |

Suivre les logs en direct sur macOS :

```bash
tail -f ~/Library/Logs/com.rewrit.app/rewrit.log
```

Chaque appel IA y laisse une ligne `LLM request`, puis `LLM response: HTTP …` ou `LLM error …` avec le corps de la réponse en cas d'échec. Les vérifications de mise à jour y apparaissent aussi (`update available`, `app is up to date`, `update check failed`).

### Lancer depuis le terminal

```bash
/Applications/rewrit.app/Contents/MacOS/rewrit
```

Les mêmes logs s'affichent alors dans le terminal.

### Panne d'octobre 2026

Groq a retiré `llama-3.3-70b-versatile` : chaque appel renvoyait `404 model_not_found`, l'erreur partait dans un toast de la fenêtre des réglages (souvent cachée) et l'app semblait ne rien faire. Le modèle par défaut est désormais `openai/gpt-oss-120b` (avec `reasoning_effort: low`). Pour voir les modèles disponibles avec ta clé :

```bash
curl -s https://api.groq.com/openai/v1/models -H "Authorization: Bearer $(cat ~/.rewrit/groq_api_key)" | python3 -m json.tool | grep '"id"'
```
