#!/bin/bash

# System Alert - Quick Release Script
# One-command release process

set -e

echo "⚡ System Alert - Quick Release Script"
echo "====================================="

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Get current version
CURRENT_VERSION=$(grep '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo -e "${BLUE}Current version: ${CURRENT_VERSION}${NC}"

# Ask for new version
echo -e "${YELLOW}Enter new version (current: ${CURRENT_VERSION}):${NC}"
read -r NEW_VERSION

if [ -z "$NEW_VERSION" ]; then
    echo "❌ Version cannot be empty"
    exit 1
fi

echo -e "${YELLOW}🔄 Updating version to ${NEW_VERSION}...${NC}"

# Update Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"${NEW_VERSION}\"/" Cargo.toml

# Update Cargo.lock
cargo check > /dev/null 2>&1

echo -e "${YELLOW}🔨 Building and testing...${NC}"

# Run full build and test cycle
make clean
make all

echo -e "${YELLOW}📝 Committing changes...${NC}"

# Commit version bump
git add Cargo.toml Cargo.lock
git commit -m "Bump version to ${NEW_VERSION}"

echo -e "${YELLOW}🏷️  Creating git tag...${NC}"

# Create and push tag
git tag "v${NEW_VERSION}"
git push origin main
git push origin "v${NEW_VERSION}"

echo -e "${GREEN}🎉 Release v${NEW_VERSION} initiated!${NC}"
echo -e "${BLUE}📋 What happens next:${NC}"
echo -e "   1. GitHub Actions will automatically build the release"
echo -e "   2. Distribution packages will be created"
echo -e "   3. GitHub release will be published"
echo -e "   4. Check: https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:/]\([^/]*\/[^/]*\)\.git/\1/')/actions"

echo -e "${GREEN}✨ Done! Your release is on its way!${NC}"