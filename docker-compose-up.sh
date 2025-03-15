#!/bin/bash
set -e

# Print help
if [ "$1" == "-h" ] || [ "$1" == "--help" ]; then
  echo "Usage: ./docker-compose-up.sh [dev|prod|both]"
  echo "  dev  - Start development environment only (default)"
  echo "  prod - Start production environment only"
  echo "  both - Start both development and production environments"
  exit 0
fi

ENV=${1:-dev}

echo "🚀 Starting MyHealth API - $ENV environment"

# Make sure .env file exists
if [ ! -f .env ]; then
  echo "⚠️ .env file not found. Creating from .env.example..."
  cp .env.example .env
  echo "✅ Created .env file. Please update with your settings."
fi

# Configure based on environment
case "$ENV" in
  dev)
    echo "🔧 Building and starting development environment..."
    docker-compose up -d dev
    ;;
  prod)
    echo "🔧 Building and starting production environment..."
    docker-compose up -d prod
    ;;
  both)
    echo "🔧 Building and starting both development and production environments..."
    docker-compose up -d
    ;;
  *)
    echo "❌ Invalid environment: $ENV"
    echo "Valid options: dev, prod, both"
    exit 1
    ;;
esac

# Wait for containers to be healthy
echo "⏳ Waiting for services to be ready..."
sleep 5

# Display container status
docker-compose ps

if [ "$ENV" == "dev" ] || [ "$ENV" == "both" ]; then
  echo ""
  echo "✅ Development server running at http://localhost:3000"
  echo "📝 API documentation available at http://localhost:3000/api/v1/docs"
  echo "📋 To view logs: docker-compose logs -f dev"
fi

if [ "$ENV" == "prod" ] || [ "$ENV" == "both" ]; then
  echo ""
  echo "✅ Production server running at http://localhost:3001"
  echo "📝 API documentation available at http://localhost:3001/api/v1/docs"
  echo "📋 To view logs: docker-compose logs -f prod"
fi

echo ""
echo "📊 To check container status: docker-compose ps"
echo "🛑 To stop services: docker-compose down" 