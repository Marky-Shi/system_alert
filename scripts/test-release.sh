#!/bin/bash

# System Alert - Release Testing Script
# Tests the built release binary

set -e

echo "🧪 System Alert - Release Testing Script"
echo "========================================"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

PROJECT_NAME="system-alert"
BINARY_PATH="target/release/${PROJECT_NAME}"

if [ ! -f "${BINARY_PATH}" ]; then
    echo -e "${RED}❌ Release binary not found. Run build-release.sh first.${NC}"
    exit 1
fi

echo -e "${YELLOW}🔍 Testing release binary...${NC}"

# Test 1: Binary exists and is executable
echo -e "${YELLOW}Test 1: Binary executable check...${NC}"
if [ -x "${BINARY_PATH}" ]; then
    echo -e "${GREEN}✅ Binary is executable${NC}"
else
    echo -e "${RED}❌ Binary is not executable${NC}"
    exit 1
fi

# Test 2: Help command
echo -e "${YELLOW}Test 2: Help command...${NC}"
if "${BINARY_PATH}" --help > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Help command works${NC}"
else
    echo -e "${RED}❌ Help command failed${NC}"
    exit 1
fi

# Test 3: Version command
echo -e "${YELLOW}Test 3: Version command...${NC}"
if "${BINARY_PATH}" --version > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Version command works${NC}"
else
    echo -e "${RED}❌ Version command failed${NC}"
    exit 1
fi

# Test 4: Binary size check
echo -e "${YELLOW}Test 4: Binary size check...${NC}"
BINARY_SIZE=$(stat -f%z "${BINARY_PATH}")
if [ "${BINARY_SIZE}" -gt 1000000 ]; then  # > 1MB
    echo -e "${GREEN}✅ Binary size reasonable: $(du -h "${BINARY_PATH}" | cut -f1)${NC}"
else
    echo -e "${RED}❌ Binary size too small: $(du -h "${BINARY_PATH}" | cut -f1)${NC}"
    exit 1
fi

# Test 5: Dependencies check
echo -e "${YELLOW}Test 5: Dependencies check...${NC}"
if otool -L "${BINARY_PATH}" | grep -q "libSystem"; then
    echo -e "${GREEN}✅ System dependencies look good${NC}"
else
    echo -e "${RED}❌ Missing system dependencies${NC}"
    exit 1
fi

# Test 6: Quick run test (if running as root)
echo -e "${YELLOW}Test 6: Quick run test...${NC}"
if [ "$EUID" -eq 0 ]; then
    echo -e "${YELLOW}Running as root, testing quick startup...${NC}"
    timeout 3s "${BINARY_PATH}" > /dev/null 2>&1 || true
    echo -e "${GREEN}✅ Quick run test completed${NC}"
else
    echo -e "${YELLOW}⚠️  Not running as root, skipping runtime test${NC}"
    echo -e "${YELLOW}   To test fully, run: sudo ${BINARY_PATH}${NC}"
fi

echo -e "${GREEN}🎉 All tests passed! Release binary is ready.${NC}"

# Show binary info
echo -e "${YELLOW}📊 Binary Information:${NC}"
echo -e "   Path: ${BINARY_PATH}"
echo -e "   Size: $(du -h "${BINARY_PATH}" | cut -f1)"
echo -e "   Arch: $(file "${BINARY_PATH}" | cut -d: -f2)"
echo -e "   Modified: $(stat -f%Sm "${BINARY_PATH}")"

echo -e "${GREEN}✨ Ready for distribution!${NC}"