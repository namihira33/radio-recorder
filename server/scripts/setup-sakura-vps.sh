#!/bin/bash
# Sakura VPS Setup Script for Radio Recorder
# Tested on Ubuntu 22.04 / Debian 12

set -e

echo "=== Radio Recorder VPS Setup ==="
echo ""

# Update system
echo "[1/5] Updating system packages..."
sudo apt-get update
sudo apt-get upgrade -y

# Install Docker
echo "[2/5] Installing Docker..."
if ! command -v docker &> /dev/null; then
    curl -fsSL https://get.docker.com -o get-docker.sh
    sudo sh get-docker.sh
    sudo usermod -aG docker $USER
    rm get-docker.sh
    echo "Docker installed. Please log out and log back in, then run this script again."
    exit 0
fi

# Install Docker Compose
echo "[3/5] Installing Docker Compose..."
if ! command -v docker-compose &> /dev/null; then
    sudo apt-get install -y docker-compose-plugin
fi

# Create app directory
echo "[4/5] Setting up application..."
APP_DIR=~/radio-recorder
mkdir -p $APP_DIR
cd $APP_DIR

# Clone or update repository
if [ -d ".git" ]; then
    git pull
else
    git clone https://github.com/YOUR_USERNAME/radio-recorder.git .
fi

# Create data directories
mkdir -p data recordings

# Start the application
echo "[5/5] Starting Radio Recorder..."
cd server
docker compose up -d --build

echo ""
echo "=== Setup Complete! ==="
echo ""
echo "Radio Recorder is now running!"
echo "Access the web interface at: http://YOUR_VPS_IP:3000"
echo ""
echo "Useful commands:"
echo "  View logs:     docker compose logs -f"
echo "  Stop:          docker compose down"
echo "  Restart:       docker compose restart"
echo "  Update:        git pull && docker compose up -d --build"
echo ""
echo "Recordings are stored in: $APP_DIR/recordings"
