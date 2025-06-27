#!/bin/bash

# GitHub CLI Authentication Setup Script
# This script helps set up GitHub CLI authentication that persists across container rebuilds

set -e

echo "🔐 GitHub CLI Authentication Setup"
echo "=================================="
echo

# Check if already authenticated
if gh auth status &>/dev/null; then
    echo "✅ GitHub CLI is already authenticated!"
    gh auth status
    exit 0
fi

echo "Setting up GitHub CLI authentication with persistent credentials..."
echo

# Check if GH_TOKEN environment variable is set
if [ -n "$GH_TOKEN" ]; then
    echo "🔑 Found GH_TOKEN environment variable"
    echo "Testing token authentication..."

    if echo "$GH_TOKEN" | gh auth login --with-token; then
        echo "✅ Successfully authenticated with GH_TOKEN!"
        gh auth status
        exit 0
    else
        echo "❌ Failed to authenticate with GH_TOKEN"
        echo "The token may be invalid or expired"
        echo
    fi
fi

# Interactive authentication
echo "🌐 Starting interactive GitHub CLI authentication..."
echo
echo "This will:"
echo "1. Open your browser to authenticate with GitHub"
echo "2. Store credentials in ~/.config/gh (persisted on host)"
echo "3. Allow access to GitHub APIs and repositories"
echo
echo "Required scopes: repo, read:org, gist"
echo

read -p "Continue with interactive authentication? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Authentication cancelled."
    exit 1
fi

# Run interactive authentication
gh auth login --git-protocol https --web

# Verify authentication
if gh auth status &>/dev/null; then
    echo
    echo "✅ GitHub CLI authentication successful!"
    echo "Credentials are stored in ~/.config/gh and will persist across container rebuilds."
    echo
    gh auth status
else
    echo
    echo "❌ Authentication failed. Please try again."
    exit 1
fi

echo
echo "🎉 Setup complete! You can now use 'gh' commands to interact with GitHub."
echo "Examples:"
echo "  gh run list                    # View workflow runs"
echo "  gh run view --log <run-id>     # View workflow logs"
echo "  gh pr list                     # List pull requests"
echo "  gh issue list                  # List issues"
