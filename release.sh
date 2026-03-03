#!/bin/bash

PACKAGE_VERSION=$(cat ./crates/move-cli/Cargo.toml | grep version | head -n 1 | sed -e 's/"//g' | cut -d ' '  -f3)
RED="\033[0;31m"
NC="\033[0m"

echo "Releasing move-stylus version $PACKAGE_VERSION"

# Creating a git tag for the release
GIT_TAG="v$PACKAGE_VERSION"

if git rev-parse "$GIT_TAG" >/dev/null 2>&1; then
    echo -e "${RED}Error${NC}: Git tag $GIT_TAG already exists. Did you forget to update the version number in Cargo.toml?"
    exit 1
fi

git tag -a "$GIT_TAG" -m "Release version $PACKAGE_VERSION"
git push origin "$GIT_TAG"

echo "Release $PACKAGE_VERSION created successfully with git tag $GIT_TAG."