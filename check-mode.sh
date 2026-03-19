#!/bin/bash

# Script pour vérifier le mode de fonctionnement actuel

# Couleurs
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}╔══════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   rewrit - Configuration Check      ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════╝${NC}"
echo ""

# Vérifier si .env existe
if [ ! -f "backend/.env" ]; then
    echo -e "${RED}❌ Fichier backend/.env non trouvé${NC}"
    echo ""
    echo "Créez-le depuis .env.example:"
    echo "  cp backend/.env.example backend/.env"
    exit 1
fi

# Lire les variables
USE_AI=$(grep "^USE_AI=" backend/.env | cut -d '=' -f2 | tr -d ' ')
API_KEY=$(grep "^ANTHROPIC_API_KEY=" backend/.env | cut -d '=' -f2)
DB_URL=$(grep "^DATABASE_URL=" backend/.env | cut -d '=' -f2)

echo -e "${BLUE}Configuration actuelle:${NC}"
echo ""

# Afficher USE_AI
echo -n "USE_AI: "
if [ "$USE_AI" = "true" ]; then
    echo -e "${GREEN}true${NC} (AI activée)"
else
    echo -e "${YELLOW}false${NC} (mode mock)"
fi

# Afficher API Key
echo -n "API Key: "
if [ -z "$API_KEY" ] || [ "$API_KEY" = "sk-ant-xxxxx" ]; then
    echo -e "${RED}Non configurée${NC}"
else
    echo -e "${GREEN}Configurée${NC} (${API_KEY:0:10}...)"
fi

# Afficher Database
echo -n "Database: "
if [ -z "$DB_URL" ]; then
    echo -e "${RED}Non configurée${NC}"
else
    echo -e "${GREEN}Configurée${NC}"
fi

echo ""
echo -e "${BLUE}Mode de fonctionnement:${NC}"
echo ""

# Déterminer le mode
if [ "$USE_AI" = "true" ]; then
    if [ -z "$API_KEY" ] || [ "$API_KEY" = "sk-ant-xxxxx" ]; then
        echo -e "${RED}❌ ERREUR: AI activée mais pas de clé API${NC}"
        echo ""
        echo "Solutions:"
        echo "  1. Mettez USE_AI=false dans backend/.env (mode mock)"
        echo "  2. Ou ajoutez votre clé API Claude dans ANTHROPIC_API_KEY"
    else
        echo -e "${GREEN}✅ MODE AI ACTIVÉ${NC}"
        echo ""
        echo "L'application utilise l'API Claude pour les traductions"
        echo "⚠️  Attention aux coûts d'API"
    fi
else
    echo -e "${YELLOW}🧪 MODE MOCK ACTIVÉ${NC}"
    echo ""
    echo "L'application utilise des traductions hardcodées"
    echo "✅ Pas de coûts d'API"
    echo "✅ Fonctionne sans connexion internet"
    echo ""
    echo "Traductions disponibles:"
    echo "  - hello → bonjour"
    echo "  - thank you → merci"
    echo "  - goodbye → au revoir"
    echo "  - etc. (voir FEATURE_FLAGS.md)"
    echo ""
    echo "Pour activer l'IA:"
    echo "  1. Éditez backend/.env"
    echo "  2. Changez USE_AI=true"
    echo "  3. Ajoutez votre ANTHROPIC_API_KEY"
    echo "  4. Redémarrez le backend"
fi

echo ""
echo -e "${BLUE}Actions rapides:${NC}"
echo ""
echo "Basculer en mode AI:"
echo "  sed -i '' 's/USE_AI=false/USE_AI=true/' backend/.env"
echo ""
echo "Basculer en mode Mock:"
echo "  sed -i '' 's/USE_AI=true/USE_AI=false/' backend/.env"
echo ""
echo "Vérifier le statut de l'API (si le backend tourne):"
echo "  curl http://localhost:8000/api/v1/status"
echo ""
