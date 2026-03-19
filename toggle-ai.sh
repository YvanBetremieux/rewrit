#!/bin/bash

# Script pour basculer rapidement entre mode AI et Mock

# Couleurs
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

ENV_FILE="backend/.env"

if [ ! -f "$ENV_FILE" ]; then
    echo -e "${RED}❌ Fichier $ENV_FILE non trouvé${NC}"
    exit 1
fi

# Lire la valeur actuelle
CURRENT=$(grep "^USE_AI=" "$ENV_FILE" | cut -d '=' -f2 | tr -d ' ')

echo -e "${BLUE}╔══════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   rewrit - Toggle AI Mode            ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════╝${NC}"
echo ""

# Afficher le mode actuel
echo -n "Mode actuel: "
if [ "$CURRENT" = "true" ]; then
    echo -e "${GREEN}AI ACTIVÉ${NC}"
    NEW_MODE="false"
    NEW_MODE_NAME="Mock"
else
    echo -e "${YELLOW}MOCK${NC}"
    NEW_MODE="true"
    NEW_MODE_NAME="AI"
fi

echo ""
echo -e "Voulez-vous basculer en mode ${BLUE}${NEW_MODE_NAME}${NC}?"
echo ""

if [ "$NEW_MODE" = "true" ]; then
    # Vérifier si une API key est configurée
    API_KEY=$(grep "^ANTHROPIC_API_KEY=" "$ENV_FILE" | cut -d '=' -f2)
    if [ -z "$API_KEY" ] || [ "$API_KEY" = "sk-ant-xxxxx" ]; then
        echo -e "${YELLOW}⚠️  Attention: Aucune clé API configurée${NC}"
        echo ""
        echo "Le mode AI nécessite une clé API Claude."
        echo "1. Obtenez une clé sur: https://console.anthropic.com/"
        echo "2. Ajoutez-la dans $ENV_FILE"
        echo ""
        read -p "Continuer quand même? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Annulé."
            exit 0
        fi
    else
        echo -e "${GREEN}✅ Clé API détectée${NC}"
    fi
    echo ""
    echo "Mode AI: Utilise l'API Claude"
    echo "⚠️  Attention aux coûts d'API"
else
    echo "Mode Mock: Utilise des traductions hardcodées"
    echo "✅ Pas de coûts"
    echo "✅ Traductions: hello→bonjour, merci→thank you, etc."
fi

echo ""
read -p "Confirmer? (y/N) " -n 1 -r
echo

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Annulé."
    exit 0
fi

# Basculer
if [ "$NEW_MODE" = "true" ]; then
    sed -i '' 's/USE_AI=false/USE_AI=true/' "$ENV_FILE"
else
    sed -i '' 's/USE_AI=true/USE_AI=false/' "$ENV_FILE"
fi

echo ""
echo -e "${GREEN}✅ Mode changé avec succès!${NC}"
echo ""

# Afficher le nouveau mode
if [ "$NEW_MODE" = "true" ]; then
    echo -e "${GREEN}🤖 MODE AI ACTIVÉ${NC}"
    echo ""
    echo "L'API utilisera Claude pour les traductions"
else
    echo -e "${YELLOW}🧪 MODE MOCK ACTIVÉ${NC}"
    echo ""
    echo "L'API utilisera des traductions hardcodées"
fi

echo ""
echo -e "${BLUE}Prochaine étape:${NC}"
echo "Redémarrez le backend pour appliquer les changements"
echo ""
echo "Si vous utilisez dev-start.sh, appuyez sur Ctrl+C et relancez:"
echo "  ./dev-start.sh"
echo ""
echo "Si vous avez lancé manuellement:"
echo "  cd backend"
echo "  source venv/bin/activate"
echo "  python main.py"
echo ""
