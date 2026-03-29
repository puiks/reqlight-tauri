#!/usr/bin/env bash
set -euo pipefail

# Usage: ./scripts/bump.sh <version>
# Example: ./scripts/bump.sh 0.5.0
#
# This script:
#   1. Updates version in package.json, Cargo.toml, tauri.conf.json
#   2. Updates Cargo.lock via cargo check
#   3. Stamps [Unreleased] → [version] - today's date in CHANGELOG.md
#   4. Commits all changes
#   5. Creates git tag v<version>
#
# After running, push with: git push --follow-tags

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="${1:-}"

if [[ -z "$VERSION" ]]; then
  echo "Usage: $0 <version>"
  echo "Example: $0 0.5.0"
  exit 1
fi

# Validate semver format
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: Version must be semver format (e.g. 1.2.3)"
  exit 1
fi

TAG="v${VERSION}"

# Check tag doesn't already exist
if git -C "$ROOT" tag -l "$TAG" | grep -q "$TAG"; then
  echo "Error: Tag $TAG already exists"
  exit 1
fi

# Check working tree is clean (except Cargo.lock which we'll regenerate)
if [[ -n "$(git -C "$ROOT" status --porcelain -- ':!src-tauri/Cargo.lock')" ]]; then
  echo "Error: Working tree has uncommitted changes. Commit or stash first."
  exit 1
fi

echo "Bumping version to $VERSION..."

# Cross-platform sed in-place (macOS uses -i '', GNU uses -i)
sedi() {
  if sed --version 2>/dev/null | grep -q GNU; then
    sed -i "$@"
  else
    sed -i '' "$@"
  fi
}

# 1. Update package.json
sedi "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" "$ROOT/package.json"

# 2. Update Cargo.toml (only the first version line under [package])
sedi "0,/^version = \".*\"/s//version = \"$VERSION\"/" "$ROOT/src-tauri/Cargo.toml"

# 3. Update tauri.conf.json
sedi "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" "$ROOT/src-tauri/tauri.conf.json"

# 4. Update Cargo.lock
(cd "$ROOT/src-tauri" && cargo check --quiet 2>/dev/null || true)

# 5. Stamp CHANGELOG.md: [Unreleased] → [version] - date, add new [Unreleased]
TODAY="$(date +%Y-%m-%d)"
sedi "s/^## \[Unreleased\]/## [Unreleased]\n\n## [$VERSION] - $TODAY/" "$ROOT/CHANGELOG.md"

echo "Updated:"
echo "  package.json        → $VERSION"
echo "  Cargo.toml          → $VERSION"
echo "  tauri.conf.json     → $VERSION"
echo "  CHANGELOG.md        → [$VERSION] - $TODAY"

# 6. Commit & tag
git -C "$ROOT" add \
  package.json \
  src-tauri/Cargo.toml \
  src-tauri/Cargo.lock \
  src-tauri/tauri.conf.json \
  CHANGELOG.md

git -C "$ROOT" commit -m "$(cat <<EOF
🔖 - Bump version to $VERSION
EOF
)"

git -C "$ROOT" tag -a "$TAG" -m "Release $TAG"

echo ""
echo "Done! Created commit and tag $TAG"
echo "Run 'git push --follow-tags' to trigger the release workflow."
