#!/bin/bash

# rewrit Quick Start Script
# This script helps you set up and run rewrit MVP 1

set -e

echo "🚀 rewrit MVP 1 - Quick Start"
echo "=============================="
echo ""

# Check prerequisites
echo "📋 Checking prerequisites..."

# Check Node.js
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed"
    echo "   Please install from: https://nodejs.org/"
    exit 1
fi
echo "✅ Node.js $(node --version)"

# Check npm
if ! command -v npm &> /dev/null; then
    echo "❌ npm is not installed"
    exit 1
fi
echo "✅ npm $(npm --version)"

# Check Python
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 is not installed"
    echo "   Please install from: https://www.python.org/"
    exit 1
fi
echo "✅ Python $(python3 --version)"

# Check Rust
if ! command -v rustc &> /dev/null; then
    echo "⚠️  Rust is not installed"
    echo "   Install with: curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh"
    echo "   Then restart your terminal and run this script again"
    exit 1
fi
echo "✅ Rust $(rustc --version)"

# Check Cargo
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo is not installed"
    exit 1
fi
echo "✅ Cargo $(cargo --version)"

# Check PostgreSQL
if ! command -v psql &> /dev/null; then
    echo "⚠️  PostgreSQL is not installed"
    echo "   macOS: brew install postgresql@15"
    echo "   Then: brew services start postgresql@15"
    exit 1
fi
echo "✅ PostgreSQL installed"

echo ""
echo "🔧 Setting up backend..."

# Create backend virtual environment if it doesn't exist
if [ ! -d "backend/venv" ]; then
    echo "Creating Python virtual environment..."
    cd backend
    python3 -m venv venv
    cd ..
fi

# Activate venv and install dependencies
echo "Installing Python dependencies..."
cd backend
source venv/bin/activate
pip install -q -r requirements.txt
cd ..

# Check if .env exists
if [ ! -f "backend/.env" ]; then
    echo ""
    echo "⚠️  backend/.env not found"
    echo "Creating from .env.example..."
    cp backend/.env.example backend/.env
    echo ""
    echo "❗ IMPORTANT: Edit backend/.env and configure your database URL"
    echo "   AI (Groq) is configured in the app Settings"
    echo ""
    read -p "Press Enter to open .env in nano, or Ctrl+C to edit manually..."
    nano backend/.env || true
fi

# Check if database exists
echo ""
echo "🗄️  Checking database..."
if psql -lqt | cut -d \| -f 1 | grep -qw rewrit; then
    echo "✅ Database 'rewrit' exists"
else
    echo "Creating database 'rewrit'..."
    createdb rewrit || {
        echo "⚠️  Could not create database. You may need to create it manually:"
        echo "   createdb rewrit"
    }
fi

# Install Node.js dependencies if needed
if [ ! -d "node_modules" ]; then
    echo ""
    echo "📦 Installing Node.js dependencies..."
    npm install
fi

echo ""
echo "✅ Setup complete!"
echo ""
echo "🎯 To run rewrit:"
echo ""
echo "Terminal 1 - Backend API:"
echo "  cd backend"
echo "  source venv/bin/activate"
echo "  python main.py"
echo ""
echo "Terminal 2 - Desktop App:"
echo "  npm run tauri:dev"
echo ""
echo "💡 Tip: The first build may take a while (Rust compilation)"
echo ""
echo "📚 For more information, see:"
echo "  - README.md - Complete documentation"
echo "  - IMPLEMENTATION_SUMMARY.md - Implementation details"
echo "  - DoD.md - Full specifications"
echo ""
echo "🚀 Happy coding!"
