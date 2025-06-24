#!/bin/bash

# Setup GitHub CLI authentication for the semantic search project
# Run this inside the devcontainer to authenticate with GitHub

echo "🐙 Setting up GitHub CLI authentication..."

# Check if gh is installed
if ! command -v gh >/dev/null 2>&1; then
    echo "❌ GitHub CLI not found. Installing..."
    
    # Install gh if not available
    curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
    sudo chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
    sudo apt update
    sudo apt install gh -y
fi

echo "✅ GitHub CLI found"

# Check if already authenticated
if gh auth status >/dev/null 2>&1; then
    echo "✅ Already authenticated with GitHub!"
    gh auth status
    exit 0
fi

echo ""
echo "🔐 Setting up GitHub authentication..."
echo ""
echo "Choose your authentication method:"
echo "1. Login via web browser (recommended)"
echo "2. Login with personal access token"
echo ""

read -p "Enter your choice (1 or 2): " auth_choice

case $auth_choice in
    1)
        echo "🌐 Opening browser for authentication..."
        gh auth login --web
        ;;
    2)
        echo "🔑 Enter your personal access token:"
        echo "   (Get one from: https://github.com/settings/tokens)"
        gh auth login --with-token
        ;;
    *)
        echo "Invalid choice. Run the script again."
        exit 1
        ;;
esac

# Verify authentication
echo ""
echo "🔍 Verifying authentication..."
if gh auth status; then
    echo ""
    echo "🎉 GitHub CLI authentication successful!"
    echo ""
    echo "📋 You can now:"
    echo "  - Clone private repos: gh repo clone owner/repo"
    echo "  - Create repos: gh repo create"
    echo "  - Manage issues and PRs: gh pr list, gh issue list"
    echo "  - Your auth will persist between container rebuilds"
    echo ""
    echo "🔧 For the semantic search project:"
    echo "  - Push to GitHub: git push"
    echo "  - Create releases: gh release create"
    echo "  - Manage project issues: gh issue create"
else
    echo "❌ Authentication failed. Please try again."
    exit 1
fi 