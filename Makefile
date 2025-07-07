# System Alert - Makefile
# Automated build and release management

.PHONY: help build test release clean install uninstall github-release

# Default target
help:
	@echo "🚀 System Alert - Build System"
	@echo "=============================="
	@echo ""
	@echo "Available targets:"
	@echo "  build          - Build debug version"
	@echo "  release        - Build optimized release version"
	@echo "  test           - Run tests"
	@echo "  test-release   - Test the release binary"
	@echo "  clean          - Clean build artifacts"
	@echo "  install        - Install locally (requires sudo)"
	@echo "  uninstall      - Uninstall from system"
	@echo "  package        - Create distribution package"
	@echo "  github-release - Create GitHub release (requires gh CLI)"
	@echo "  all            - Build, test, and package"
	@echo ""
	@echo "Examples:"
	@echo "  make release        # Build optimized binary"
	@echo "  make package        # Create distribution package"
	@echo "  make github-release # Create GitHub release"

# Build debug version
build:
	@echo "🔨 Building debug version..."
	cargo build

# Build release version
release:
	@echo "🔨 Building release version..."
	cargo build --release

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test

# Test release binary
test-release: release
	@echo "🧪 Testing release binary..."
	./scripts/test-release.sh

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	rm -rf release-builds dist

# Install locally
install: release
	@echo "📦 Installing locally..."
	sudo cp target/release/system-alert /usr/local/bin/
	sudo chmod +x /usr/local/bin/system-alert
	@echo "✅ Installed to /usr/local/bin/system-alert"
	@echo "🎯 Run with: sudo system-alert"

# Uninstall from system
uninstall:
	@echo "🗑️  Uninstalling..."
	sudo rm -f /usr/local/bin/system-alert
	@echo "✅ Uninstalled successfully"

# Create distribution package
package: release test-release
	@echo "📦 Creating distribution package..."
	./scripts/build-release.sh

# Create GitHub release
github-release: package
	@echo "🐙 Creating GitHub release..."
	./scripts/github-release.sh

# Build everything
all: clean build test release test-release package
	@echo "✨ All tasks completed successfully!"

# Development helpers
dev-run: build
	@echo "🚀 Running development version..."
	sudo target/debug/system-alert

release-run: release
	@echo "🚀 Running release version..."
	sudo target/release/system-alert

# Check dependencies
check-deps:
	@echo "🔍 Checking dependencies..."
	@command -v cargo >/dev/null 2>&1 || { echo "❌ Rust/Cargo not installed"; exit 1; }
	@command -v git >/dev/null 2>&1 || { echo "❌ Git not installed"; exit 1; }
	@echo "✅ Dependencies OK"

# Show version info
version:
	@echo "📊 Version Information:"
	@grep '^version' Cargo.toml
	@echo "Git commit: $$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
	@echo "Build date: $$(date)"

# Quick development cycle
dev: clean build test
	@echo "🔄 Development cycle complete"

# Release cycle
release-cycle: clean build test release test-release package
	@echo "🚀 Release cycle complete"