#!/bin/bash
# Deploy rewrit backend to Oracle Cloud VPS
set -e

VPS_IP="141.253.99.48"
SSH_KEY="$HOME/Downloads/ssh-key-2026-03-21.key"
BACKEND_DIR="/home/ubuntu/rewrit-backend"

echo "Deploying backend to $VPS_IP..."

# Copy backend files
scp -i "$SSH_KEY" backend/main.py backend/config.py backend/database.py backend/models.py backend/requirements.txt "ubuntu@$VPS_IP:$BACKEND_DIR/"

# Install deps + restart service
ssh -i "$SSH_KEY" "ubuntu@$VPS_IP" "cd $BACKEND_DIR && source venv/bin/activate && pip install -q -r requirements.txt && sudo systemctl restart rewrit"

echo "Done! Backend restarted on $VPS_IP"
