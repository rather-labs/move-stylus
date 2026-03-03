#!/bin/bash
# Script to update SHA256 checksums in the Homebrew formula after building releases
# Creates both the main formula (latest) and a versioned formula

set -e

FORMULA_DIR="Formula"
TEMPLATE_FILE="$FORMULA_DIR/move-stylus.rb.template"
MAIN_FORMULA="$FORMULA_DIR/move-stylus.rb"
CARGO_TOML="./crates/move-cli/Cargo.toml"

# Extract version from Cargo.toml
VERSION=$(grep version "$CARGO_TOML" | head -n 1 | awk '{print $3}' | sed -e 's/"//g')

# Create versioned formula name using full version
VERSIONED_FORMULA="$FORMULA_DIR/move-stylus@${VERSION}.rb"

# Create class name for versioned formula (e.g., MoveStylusAT010)
CLASS_NAME_VERSIONED="MoveStylusAT$(echo $VERSION | sed 's/\.//g')"

echo "=========================================="
echo "Updating Homebrew formulae for version: $VERSION"
echo "  Versioned formula: $VERSIONED_FORMULA"
echo "  Versioned class: $CLASS_NAME_VERSIONED"
echo "=========================================="
echo ""

# Check if template exists
if [ ! -f "$TEMPLATE_FILE" ]; then
    echo "Error: Template file $TEMPLATE_FILE not found"
    exit 1
fi

# Check if dist directory exists
if [ ! -d "dist" ]; then
    echo "Error: dist directory not found. Please run 'make release-*' commands first."
    exit 1
fi

# Function to extract SHA256 from .sha256 file
get_sha256() {
    local sha_file=$1
    if [ -f "$sha_file" ]; then
        awk '{print $1}' "$sha_file"
    else
        echo "SHA256 file not found: $sha_file" >&2
        echo ""
    fi
}

# Get SHA256 checksums
MACOS_SHA=$(get_sha256 "dist/move-stylus-aarch64-macos-${VERSION}.tar.gz.sha256")
LINUX_ARM64_SHA=$(get_sha256 "dist/move-stylus-aarch64-linux-${VERSION}.tar.gz.sha256")
LINUX_X86_64_SHA=$(get_sha256 "dist/move-stylus-x86_64-linux-${VERSION}.tar.gz.sha256")

echo "Found checksums:"
echo "  macOS ARM64:   $MACOS_SHA"
echo "  Linux ARM64:   $LINUX_ARM64_SHA"
echo "  Linux x86_64:  $LINUX_X86_64_SHA"
echo ""

# Validate we have all checksums
if [ -z "$MACOS_SHA" ] || [ -z "$LINUX_ARM64_SHA" ] || [ -z "$LINUX_X86_64_SHA" ]; then
    echo "Warning: Some checksums are missing. Please ensure all release binaries are built."
fi

# Function to create formula from template
create_formula_from_template() {
    local output_file=$1
    local class_name=$2
    
    echo "Creating $output_file from template..."
    
    # Use sed to replace placeholders in template
    sed -e "s/{{CLASS_NAME}}/$class_name/g" \
        -e "s/{{VERSION}}/$VERSION/g" \
        -e "s/{{MACOS_SHA256}}/$MACOS_SHA/g" \
        -e "s/{{LINUX_ARM64_SHA256}}/$LINUX_ARM64_SHA/g" \
        -e "s/{{LINUX_X86_64_SHA256}}/$LINUX_X86_64_SHA/g" \
        "$TEMPLATE_FILE" > "$output_file"
    
    echo "✓ Created $output_file"
}

# Create main formula
create_formula_from_template "$MAIN_FORMULA" "MoveStylus"

echo ""

# Create versioned formula
create_formula_from_template "$VERSIONED_FORMULA" "$CLASS_NAME_VERSIONED"

echo ""
echo "=========================================="
echo "✓ Successfully updated Homebrew formulae!"
echo "=========================================="
echo ""
echo "Updated files:"
echo "  - $MAIN_FORMULA (latest version: $VERSION)"
echo "  - $VERSIONED_FORMULA (specific version: $VERSION)"
echo ""
