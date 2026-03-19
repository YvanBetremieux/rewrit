# Configuration des Délais

Tous les délais (sleep/wait) dans l'application peuvent être configurés via un fichier `.env` dans le répertoire `src-tauri/`.

## Utilisation

1. Créez un fichier `.env` dans le répertoire `src-tauri/`
2. Ajoutez les variables d'environnement que vous souhaitez personnaliser
3. Les valeurs sont en millisecondes

## Variables disponibles

```env
# Délai après avoir caché la fenêtre de prévisualisation avant de coller
DELAY_WINDOW_HIDE=200

# Délai avant de restaurer le texte original dans le presse-papiers
DELAY_RESTORE_CLIPBOARD=500

# Délai après avoir écrit dans le presse-papiers avant de coller (mode preview)
DELAY_CLIPBOARD_READY=150

# Délai entre les pressions de touches du clavier (opérations Cmd/Ctrl)
DELAY_KEYBOARD_KEY_PRESS=100

# Délai court pour les opérations de clavier (utilisé pour Cmd+X/Cmd+C)
DELAY_KEYBOARD_SHORT_DELAY=50

# Délai après l'opération cut/copy (mode preview)
DELAY_CUT_OPERATION=200

# Délai après l'opération cut en mode direct paste (plus long pour s'assurer que le texte est coupé)
DELAY_CUT_OPERATION_DIRECT_PASTE=300

# Délai pour la vérification du presse-papiers après avoir écrit le texte traduit
DELAY_CLIPBOARD_VERIFICATION=100

# Délai après avoir écrit dans le presse-papiers avant de coller (mode direct paste)
DELAY_CLIPBOARD_READY_DIRECT_PASTE=300

# Délai entre les tentatives de lecture du presse-papiers
DELAY_CLIPBOARD_RETRY=100

# Délai dans AppleScript (en millisecondes, converti en secondes pour AppleScript)
DELAY_APPLESCRIPT_DELAY=200
```

## Valeurs par défaut

Si une variable n'est pas définie dans le fichier `.env`, la valeur par défaut (indiquée ci-dessus) sera utilisée.

## Exemple de fichier .env

```env
# Personnaliser les délais pour un système plus lent
DELAY_WINDOW_HIDE=300
DELAY_CLIPBOARD_READY=200
DELAY_KEYBOARD_KEY_PRESS=150
```

## Notes

- Les valeurs sont en millisecondes
- Le fichier `.env` est chargé au démarrage de l'application
- Les modifications du fichier `.env` nécessitent un redémarrage de l'application pour prendre effet

