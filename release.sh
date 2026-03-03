#!/bin/bash

PACKAGE_VERSION=$(cat ./crates/move-cli/Cargo.toml | grep version | head -n 1 | awk '{print $$3}' | sed -e 's/"//g' | cut -d ' '  -f3)
RED="\033[0;31m"
NC="\033[0m"

echo "Releasing move-cli version $PACKAGE_VERSION"

# Check that the artifacts and SHA256 checksums are in place

MACOS_ARTIFACT="./dist/move-stylus-aarch64-macos-${PACKAGE_VERSION}.tar.gz"
if [ ! -f "$MACOS_ARTIFACT" ]; then
    echo -e "${RED}Error${NC}: Artifact for macOS is missing: $MACOS_ARTIFACT"
    exit 1
fi

MACOS_CHECKSUM="./dist/move-stylus-aarch64-macos-${PACKAGE_VERSION}.tar.gz.sha256"
if [ ! -f "$MACOS_CHECKSUM" ]; then
    echo -e "${RED}Error${NC}: Checksum for macOS' artifact is missing: $MACOS_CHECKSUM"
    exit 1
fi

LINUX_X86_ARTIFACT="./dist/move-stylus-x86_64-linux-${PACKAGE_VERSION}.tar.gz"
if [ ! -f "$LINUX_X86_ARTIFACT" ]; then
    echo -e "${RED}Error${NC}: Artifact for Linux x86_64 is missing: $LINUX_X86_ARTIFACT"
    exit 1
fi

LINUX_X86_CHECKSUM="./dist/move-stylus-x86_64-linux-${PACKAGE_VERSION}.tar.gz.sha256"
if [ ! -f "$LINUX_X86_CHECKSUM" ]; then
    echo -e "${RED}Error${NC}: Checksum for Linux x86_64 artifact is missing: $LINUX_X86_CHECKSUM"
    exit 1
fi

LINUX_ARM_ARTIFACT="./dist/move-stylus-aarch64-linux-${PACKAGE_VERSION}.tar.gz"
if [ ! -f "$LINUX_ARM_ARTIFACT" ]; then
    echo -e "${RED}Error${NC}: Artifact for Linux aarch64 is missing: $LINUX_ARM_ARTIFACT"
    exit 1
fi

LINUX_ARM_CHECKSUM="./dist/move-stylus-aarch64-linux-${PACKAGE_VERSION}.tar.gz.sha256"
if [ ! -f "$LINUX_ARM_CHECKSUM" ]; then
    echo -e "${RED}Error${NC}: Checksum for Linux aarch64 artifact is missing: $LINUX_ARM_CHECKSUM"
    exit 1
fi

# Creating a git tag for the release
GIT_TAG="v$PACKAGE_VERSION"

if git rev-parse "$GIT_TAG" >/dev/null 2>&1; then
    echo -e "${RED}Error${NC}: Git tag $GIT_TAG already exists. Did you forget to update the version number in Cargo.toml?"
    exit 1
fi

git tag -a "$GIT_TAG" -m "Release version $PACKAGE_VERSION"
git push origin "$GIT_TAG"

echo "Release $PACKAGE_VERSION created successfully with git tag $GIT_TAG."
echo "Please remember to create a GitHub release and upload the artifacts and their checksums."
