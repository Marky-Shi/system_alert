#!/bin/bash

# System Alert - GitHub Release Script
# Creates a GitHub release with assets

set -e

echo "🐙 System Alert - GitHub Release Script"
echo "======================================="

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
PROJECT_NAME="system-alert"
VERSION=$(grep '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
REPO_OWNER="yourusername"  # Change this to your GitHub username
REPO_NAME="system-alert"   # Change this to your repo name
DIST_DIR="dist"

echo -e "${BLUE}📦 Preparing GitHub release for v${VERSION}${NC}"

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo -e "${RED}❌ GitHub CLI (gh) is not installed${NC}"
    echo -e "${YELLOW}Install it with: brew install gh${NC}"
    exit 1
fi

# Check if user is authenticated
if ! gh auth status &> /dev/null; then
    echo -e "${RED}❌ Not authenticated with GitHub${NC}"
    echo -e "${YELLOW}Run: gh auth login${NC}"
    exit 1
fi

# Check if distribution files exist
ARCHIVE_FILE="${DIST_DIR}/${PROJECT_NAME}-v${VERSION}-macos.tar.gz"
CHECKSUM_FILE="${DIST_DIR}/${PROJECT_NAME}-v${VERSION}-checksums.txt"

if [ ! -f "${ARCHIVE_FILE}" ]; then
    echo -e "${RED}❌ Distribution archive not found: ${ARCHIVE_FILE}${NC}"
    echo -e "${YELLOW}Run build-release.sh first${NC}"
    exit 1
fi

if [ ! -f "${CHECKSUM_FILE}" ]; then
    echo -e "${RED}❌ Checksum file not found: ${CHECKSUM_FILE}${NC}"
    echo -e "${YELLOW}Run build-release.sh first${NC}"
    exit 1
fi

# Generate release notes
echo -e "${YELLOW}📝 Generating release notes...${NC}"

RELEASE_NOTES_FILE="release-notes-v${VERSION}.md"
cat > "${RELEASE_NOTES_FILE}" << EOF
# System Alert v${VERSION}

Advanced macOS System Monitor with real-time metrics and beautiful TUI interface.

## 🚀 Features

- **🔋 Advanced Battery Monitoring**: Real-time battery health, cycle count, and charging status
- **⚡ Apple Silicon Optimization**: E-cluster/P-cluster monitoring with detailed power metrics  
- **🎨 Beautiful TUI Interface**: Clean, organized four-quadrant layout with progress bars
- **📊 Real-time Data**: Live system metrics with configurable refresh rates
- **🔔 Smart Notifications**: Configurable threshold-based alerts
- **⚙️ Highly Configurable**: TOML-based configuration with CLI overrides

## 📋 System Requirements

- **macOS**: 10.15+ (Optimized for Apple Silicon)
- **Root Privileges**: Required for accessing system metrics via \`powermetrics\`
- **Terminal**: Any modern terminal emulator with Unicode support

## 🛠 Installation

### Quick Install
\`\`\`bash
# Download and extract
curl -L -o system-alert-v${VERSION}.tar.gz https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/v${VERSION}/${PROJECT_NAME}-v${VERSION}-macos.tar.gz
tar -xzf system-alert-v${VERSION}.tar.gz
cd system-alert-v${VERSION}

# Install
./install.sh

# Run
sudo system-alert
\`\`\`

### Manual Install
\`\`\`bash
# Make executable and copy to PATH
chmod +x system-alert
sudo cp system-alert /usr/local/bin/

# Run
sudo system-alert
\`\`\`

## 🎮 Usage

\`\`\`bash
# Default settings
sudo system-alert

# Custom refresh rate
sudo system-alert --refresh 2

# Minimal mode
sudo system-alert --minimal

# Show help
system-alert --help
\`\`\`

## 🔐 Verification

Verify the download integrity:
\`\`\`bash
shasum -a 256 -c ${PROJECT_NAME}-v${VERSION}-checksums.txt
\`\`\`

## 📊 What's New in v${VERSION}

- ✅ Complete real-time data collection (no hardcoded values)
- ✅ Advanced battery monitoring with multiple data sources
- ✅ Optimized thermal management and fan speed monitoring
- ✅ Real system health metrics (uptime, load averages)
- ✅ Professional English documentation and code
- ✅ Comprehensive build and release automation

## 🐛 Known Issues

- Requires \`sudo\` for full functionality (powermetrics access)
- Some features may not work on older macOS versions

## 📄 License

MIT License - see LICENSE file for details.

---

**Built on:** $(date)
**Commit:** $(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
EOF

echo -e "${GREEN}✅ Release notes generated: ${RELEASE_NOTES_FILE}${NC}"

# Create the release
echo -e "${YELLOW}🚀 Creating GitHub release...${NC}"

gh release create "v${VERSION}" \
    "${ARCHIVE_FILE}" \
    "${CHECKSUM_FILE}" \
    --title "System Alert v${VERSION}" \
    --notes-file "${RELEASE_NOTES_FILE}" \
    --draft

echo -e "${GREEN}🎉 GitHub release created successfully!${NC}"
echo -e "${BLUE}📋 Next steps:${NC}"
echo -e "   1. Review the draft release at: https://github.com/${REPO_OWNER}/${REPO_NAME}/releases"
echo -e "   2. Edit release notes if needed"
echo -e "   3. Publish the release when ready"

# Cleanup
rm -f "${RELEASE_NOTES_FILE}"

echo -e "${GREEN}✨ Release process complete!${NC}"