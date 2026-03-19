#!/bin/bash

# rewrit Development Launcher for macOS
# Ce script lance automatiquement le projet en développement

set -e

# Export PATH for Rust, PostgreSQL, and other tools
export PATH="/opt/homebrew/opt/postgresql@15/bin:$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:$PATH"

# Couleurs pour l'affichage
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}"
echo "╔═══════════════════════════════════════════╗"
echo "║   rewrit - Development Launcher (macOS)   ║"
echo "╔═══════════════════════════════════════════╝"
echo -e "${NC}"

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$PROJECT_DIR"

# Fonction pour afficher les erreurs
error() {
    echo -e "${RED}❌ Erreur: $1${NC}"
    exit 1
}

# Fonction pour afficher les succès
success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Fonction pour afficher les infos
info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Fonction pour afficher les warnings
warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Vérifier les prérequis
info "Vérification des prérequis..."

# Vérifier Node.js
if ! command -v node &> /dev/null; then
    error "Node.js n'est pas installé. Installez-le depuis https://nodejs.org/"
fi
success "Node.js $(node --version)"

# Vérifier npm
if ! command -v npm &> /dev/null; then
    error "npm n'est pas installé"
fi
success "npm $(npm --version)"

# Vérifier Python
if ! command -v python3 &> /dev/null; then
    error "Python 3 n'est pas installé. Installez-le depuis https://www.python.org/"
fi
success "Python $(python3 --version)"

# Vérifier PostgreSQL
if ! command -v psql &> /dev/null; then
    warning "PostgreSQL n'est pas installé ou pas dans le PATH"
    warning "Installez-le avec: brew install postgresql@15"
    read -p "Voulez-vous continuer sans PostgreSQL ? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
else
    success "PostgreSQL installé"
fi

# Vérifier Rust
if ! command -v rustc &> /dev/null; then
    warning "Rust n'est pas installé"
    warning "Installez-le avec: curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh"
    read -p "Voulez-vous continuer sans Rust ? L'app Tauri ne pourra pas être compilée. (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
    SKIP_TAURI=true
else
    success "Rust $(rustc --version)"
    SKIP_TAURI=false
fi

echo ""
info "Installation des dépendances..."

# Installer les dépendances Node.js si nécessaire
if [ ! -d "node_modules" ]; then
    info "Installation des dépendances Node.js..."
    npm install || error "Échec de l'installation des dépendances Node.js"
    success "Dépendances Node.js installées"
else
    success "Dépendances Node.js déjà installées"
fi

# Créer l'environnement virtuel Python si nécessaire
if [ ! -d "backend/venv" ]; then
    info "Création de l'environnement virtuel Python..."
    cd backend
    python3 -m venv venv || error "Échec de la création de l'environnement virtuel"
    success "Environnement virtuel créé"
    cd ..
else
    success "Environnement virtuel Python existe déjà"
fi

# Installer les dépendances Python
info "Installation des dépendances Python..."
cd backend
source venv/bin/activate
pip install -q --upgrade pip
pip install -q -r requirements.txt || error "Échec de l'installation des dépendances Python"
cd ..
success "Dépendances Python installées"

# Vérifier/créer le fichier .env
if [ ! -f "backend/.env" ]; then
    warning "Fichier .env manquant"
    info "Création de backend/.env depuis .env.example..."
    cp backend/.env.example backend/.env
    success "Fichier .env créé"
    warning "IMPORTANT: Éditez backend/.env pour configurer votre base de données"
    echo ""
    echo -e "${YELLOW}Configuration par défaut:${NC}"
    echo "  - DATABASE_URL=postgresql://localhost:5432/rewrit"
    echo ""
else
    success "Fichier .env existe"
fi

# Vérifier si la base de données existe
if command -v psql &> /dev/null; then
    if psql -lqt | cut -d \| -f 1 | grep -qw rewrit; then
        success "Base de données 'rewrit' existe"
    else
        warning "Base de données 'rewrit' n'existe pas"
        read -p "Voulez-vous la créer maintenant ? (Y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Nn]$ ]]; then
            createdb rewrit && success "Base de données créée" || warning "Impossible de créer la base de données"
        fi
    fi
fi

echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║         🚀 LANCEMENT DU PROJET            ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════╝${NC}"
echo ""

# Variables pour garder les PIDs et fenêtres
BACKEND_WINDOW_ID=""
TAURI_WINDOW_ID=""

# Fonction pour tuer les processus au signal SIGINT
cleanup() {
    echo ""
    info "Arrêt des processus..."

    # Tuer tous les processus rewrit (app Tauri)
    pkill -f "rewrit.*target/debug/rewrit" 2>/dev/null || true

    # Tuer les processus npm/node liés à tauri
    pkill -f "npm run tauri:dev" 2>/dev/null || true
    pkill -f "tauri dev" 2>/dev/null || true

    # Tuer le backend Python
    pkill -f "python.*main.py" 2>/dev/null || true
    pkill -f "uvicorn" 2>/dev/null || true

    # Tuer le serveur de documentation
    if [ -n "$DOCS_PID" ]; then
        kill "$DOCS_PID" 2>/dev/null || true
    fi

    # Fermer les fenêtres Terminal
    if [ -n "$BACKEND_WINDOW_ID" ] || [ -n "$TAURI_WINDOW_ID" ]; then
        osascript <<EOF 2>/dev/null || true
tell application "Terminal"
    set windowList to windows
    repeat with win in windowList
        set winName to custom title of win
        if winName contains "rewrit -" then
            close win
        end if
    end repeat
end tell
EOF
    fi

    success "Tous les processus arrêtés"
    exit 0
}

trap cleanup SIGINT SIGTERM

# Lancer le serveur de documentation
info "Lancement du serveur de documentation..."
python3 -m http.server 8080 --directory "$PROJECT_DIR/docs" > /dev/null 2>&1 &
DOCS_PID=$!
success "Documentation disponible sur http://localhost:8080"

# Lancer le backend dans un nouveau terminal
info "Lancement du backend API..."
BACKEND_WINDOW_ID=$(osascript <<EOF
tell application "Terminal"
    do script "export PATH=\"/opt/homebrew/opt/postgresql@15/bin:\$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:\$PATH\" && cd \"$PROJECT_DIR/backend\" && source venv/bin/activate && echo '🔧 Backend API démarré' && python main.py"
    set custom title of front window to "rewrit - Backend API"
    return id of front window
end tell
EOF
)

# Attendre un peu que le backend démarre
sleep 3

# Lancer le frontend Tauri si Rust est installé
if [ "$SKIP_TAURI" = false ]; then
    info "Lancement de l'application Tauri..."
    TAURI_WINDOW_ID=$(osascript <<EOF
tell application "Terminal"
    do script "export PATH=\"/opt/homebrew/opt/postgresql@15/bin:\$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:\$PATH\" && cd \"$PROJECT_DIR\" && echo '🎨 Application Tauri démarrée' && echo 'ℹ️  La première compilation peut prendre 5-10 minutes...' && npm run tauri:dev"
    set custom title of front window to "rewrit - Tauri App"
    return id of front window
end tell
EOF
)
else
    warning "Tauri non lancé (Rust non installé)"
    info "Vous pouvez lancer le frontend Vite uniquement avec: npm run dev"
fi

echo ""
success "Projet lancé!"
echo ""
echo -e "${BLUE}Terminaux ouverts:${NC}"
echo "  📡 Terminal 1: Backend API (http://localhost:8000)"
if [ "$SKIP_TAURI" = false ]; then
    echo "  🖥️  Terminal 2: Application Tauri (desktop app)"
fi
echo ""
echo -e "${BLUE}Endpoints disponibles:${NC}"
echo "  • http://localhost:8000 - API root"
echo "  • http://localhost:8000/api/v1/status - Statut & config"
echo "  • http://localhost:8000/docs - Documentation API (Swagger)"
echo "  • http://localhost:8080 - Documentation projet"
echo ""
echo -e "${BLUE}Raccourcis clavier (dans l'app):${NC}"
echo "  • Cmd+Alt+T - Traduire EN → FR"
echo "  • Cmd+Alt+Y - Traduire FR → EN"
echo "  • Double-tap Ctrl - Quick Select (langue + ton)"
echo "  • Cmd+, - Ouvrir les paramètres"
echo ""
echo "Press Ctrl+C dans ce terminal pour tout arrêter"
echo ""

# Garder le script en vie
while true; do
    sleep 1
done
