#!/bin/bash

# System Alert - Release Build Script
# Version: 0.1.0

set -e

echo "🚀 System Alert - Release Build Script v0.1.0"
echo "=============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="system-alert"
VERSION=$(grep '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
BUILD_DIR="release-builds"
DIST_DIR="dist"

echo -e "${BLUE}📦 Building ${PROJECT_NAME} v${VERSION}${NC}"

# Clean previous builds
echo -e "${YELLOW}🧹 Cleaning previous builds...${NC}"
rm -rf target/release
rm -rf ${BUILD_DIR}
rm -rf ${DIST_DIR}
mkdir -p ${BUILD_DIR}
mkdir -p ${DIST_DIR}

# Build optimized release
echo -e "${YELLOW}🔨 Building optimized release binary...${NC}"
cargo build --release

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Build successful!${NC}"
else
    echo -e "${RED}❌ Build failed!${NC}"
    exit 1
fi

# Get binary info
BINARY_PATH="target/release/${PROJECT_NAME}"
BINARY_SIZE=$(du -h "${BINARY_PATH}" | cut -f1)

echo -e "${GREEN}📊 Build Information:${NC}"
echo -e "   Binary: ${BINARY_PATH}"
echo -e "   Size: ${BINARY_SIZE}"
echo -e "   Version: ${VERSION}"

# Copy binary to build directory
cp "${BINARY_PATH}" "${BUILD_DIR}/"

# Create distribution package
echo -e "${YELLOW}📦 Creating distribution package...${NC}"

# Create README for distribution
cat > "${BUILD_DIR}/README.txt" << EOF
System Alert v${VERSION}
========================

Advanced macOS System Monitor

INSTALLATION:
1. Copy 'system-alert' to /usr/local/bin/ or any directory in your PATH
2. Make it executable: chmod +x system-alert
3. Run with: sudo system-alert

REQUIREMENTS:
- macOS 10.15+
- Root privileges (for powermetrics access)

USAGE:
sudo ./system-alert                    # Run with default settings
sudo ./system-alert --refresh 2       # Custom refresh rate
sudo ./system-alert --minimal         # Minimal mode
sudo ./system-alert --help           # Show help

For more information, visit: https://github.com/yourusername/system-alert

Built on: $(date)
EOF

# Create install script
cat > "${BUILD_DIR}/install.sh" << 'EOF'
#!/bin/bash

echo "🚀 Installing System Alert..."

# Check if running as root for installation
if [[ $EUID -eq 0 ]]; then
   echo "❌ Don't run the installer as root. It will ask for sudo when needed."
   exit 1
fi

# Install to /usr/local/bin
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="system-alert"

if [ -f "./${BINARY_NAME}" ]; then
    echo "📦 Installing ${BINARY_NAME} to ${INSTALL_DIR}..."
    sudo cp "./${BINARY_NAME}" "${INSTALL_DIR}/"
    sudo chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    
    echo "✅ Installation complete!"
    echo "🎯 You can now run: sudo system-alert"
    echo "💡 Or from anywhere: sudo ${INSTALL_DIR}/${BINARY_NAME}"
else
    echo "❌ Binary '${BINARY_NAME}' not found in current directory"
    exit 1
fi
EOF

chmod +x "${BUILD_DIR}/install.sh"

# Create uninstall script
cat > "${BUILD_DIR}/uninstall.sh" << 'EOF'
#!/bin/bash

echo "🗑️  Uninstalling System Alert..."

INSTALL_DIR="/usr/local/bin"
BINARY_NAME="system-alert"

if [ -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
    sudo rm "${INSTALL_DIR}/${BINARY_NAME}"
    echo "✅ System Alert uninstalled successfully"
else
    echo "ℹ️  System Alert not found in ${INSTALL_DIR}"
fi
EOF

chmod +x "${BUILD_DIR}/uninstall.sh"

# Create archive
echo -e "${YELLOW}📁 Creating distribution archive...${NC}"
cd "${BUILD_DIR}"
tar -czf "../${DIST_DIR}/${PROJECT_NAME}-v${VERSION}-macos.tar.gz" *
cd ..

# Create checksums
echo -e "${YELLOW}🔐 Generating checksums...${NC}"
cd "${DIST_DIR}"
shasum -a 256 "${PROJECT_NAME}-v${VERSION}-macos.tar.gz" > "${PROJECT_NAME}-v${VERSION}-checksums.txt"
cd ..

echo -e "${GREEN}🎉 Release build complete!${NC}"
echo -e "${BLUE}📦 Distribution files:${NC}"
echo -e "   Archive: ${DIST_DIR}/${PROJECT_NAME}-v${VERSION}-macos.tar.gz"
echo -e "   Checksums: ${DIST_DIR}/${PROJECT_NAME}-v${VERSION}-checksums.txt"
echo -e "   Build directory: ${BUILD_DIR}/"

echo -e "${YELLOW}📋 Next steps:${NC}"
echo -e "   1. Test the binary: sudo ./${BUILD_DIR}/${PROJECT_NAME}"
echo -e "   2. Install locally: cd ${BUILD_DIR} && ./install.sh"
echo -e "   3. Upload ${DIST_DIR}/${PROJECT_NAME}-v${VERSION}-macos.tar.gz for distribution"

echo -e "${GREEN}✨ Ready for release!${NC}"