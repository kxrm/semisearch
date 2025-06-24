#!/bin/bash

# Sync git configuration between host and container
# Run this to set up your git identity in the container

echo "🔧 Setting up Git configuration..."

echo ""
echo "Choose an option:"
echo "1. Enter git config manually"
echo "2. Use current host git config (if available)"
echo "3. Skip git setup"
echo ""

read -p "Enter your choice (1-3): " choice

case $choice in
    1)
        echo ""
        read -p "Enter your name: " git_name
        read -p "Enter your email: " git_email
        
        git config --global user.name "$git_name"
        git config --global user.email "$git_email"
        
        echo "✅ Git configured with:"
        echo "   Name: $git_name"
        echo "   Email: $git_email"
        ;;
    2)
        if [ -f ~/.git-host-config/.gitconfig ]; then
            cp ~/.git-host-config/.gitconfig ~/.gitconfig
            echo "✅ Copied git config from host"
        else
            echo "⚠️  No host git config found, falling back to manual setup..."
            read -p "Enter your name: " git_name
            read -p "Enter your email: " git_email
            
            git config --global user.name "$git_name"
            git config --global user.email "$git_email"
        fi
        ;;
    3)
        echo "⏭️  Skipping git setup"
        exit 0
        ;;
    *)
        echo "Invalid choice"
        exit 1
        ;;
esac

# Save to volume for next time
mkdir -p ~/.git-host-config
cp ~/.gitconfig ~/.git-host-config/.gitconfig 2>/dev/null || true

echo ""
echo "🎉 Git setup complete!"
echo "Your configuration will persist between container rebuilds."
echo ""
echo "Current git config:"
git config --global --list | grep user || echo "No user config found" 