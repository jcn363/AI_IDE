#!/bin/bash
set -e

# Fix case sensitivity in file paths
find docs/src -type f -name "*.md" -exec sed -i 's/PLUGINS\.md/plugins.md/g' {} \;
find docs/src -type f -name "*.md" -exec sed -i 's/CONTRIBUTING\.md/contributing.md/g' {} \;
find docs/src -type f -name "*.md" -exec sed -i 's/INSTALLATION\.md/installation.md/g' {} \;
find docs/src -type f -name "*.md" -exec sed -i 's/README\.md/readme.md/g' {} \;
find docs/src -type f -name "*.md" -exec sed -i 's/QUICKSTART\.md/quickstart.md/g' {} \;
find docs/src -type f -name "*.md" -exec sed -i 's/CONFIGURATION\.md/configuration.md/g' {} \;
find docs/src -type f -name "*.md" -exec sed -i 's/TROUBLESHOOTING\.md/troubleshooting.md/g' {} \;

# Fix path resolution in links
find docs/src -type f -name "*.md" -exec sed -i 's/\.\.\/features\/ai-features\.html/..\/features\/ai-features.html/g' {} \;
find docs/src -type f -name "*.md" -exec sed -i 's/\.\.\/features\/security\.html/..\/features\/security.html/g' {} \;
find docs/src -type f -name "*.md" -exec sed -i 's/\.\.\/features\/collaboration\.html/..\/features\/collaboration.html/g' {} \;
find docs/src -type f -name "*.md" -exec sed -i 's/\.\.\/development\/plugins\.html/..\/development\/plugins.html/g' {} \;
find docs/src -type f -name "*.md" -exec sed -i 's/\.\.\/getting-started\/installation\.html/..\/getting-started\/installation.html/g' {} \;
find docs/src -type f -name "*.md" -exec sed -i 's/\.\.\/user-guide\/readme\.html/..\/user-guide\/readme.html/g' {} \;
find docs/src -type f -name "*.md" -exec sed -i 's/\.\.\/development\/contributing\.html/..\/development\/contributing.html/g' {} \;
find docs/src -type f -name "*.md" -exec sed -i 's/\.\.\/api\/readme\.html/..\/api\/readme.html/g' {} \;

# Rename files to match the correct case
mv "docs/src/development/PLUGINS.md" "docs/src/development/plugins.md" 2>/dev/null || true
mv "docs/src/development/CONTRIBUTING.md" "docs/src/development/contributing.md" 2>/dev/null || true
mv "docs/src/getting-started/INSTALLATION.md" "docs/src/getting-started/installation.md" 2>/dev/null || true
mv "docs/src/user-guide/README.md" "docs/src/user-guide/readme.md" 2>/dev/null || true
mv "docs/src/getting-started/QUICKSTART.md" "docs/src/getting-started/quickstart.md" 2>/dev/null || true
mv "docs/src/getting-started/CONFIGURATION.md" "docs/src/getting-started/configuration.md" 2>/dev/null || true
mv "docs/src/user-guide/TROUBLESHOOTING.md" "docs/src/user-guide/troubleshooting.md" 2>/dev/null || true

# Fix HTML file extensions in links
find docs/src -type f -name "*.md" -exec sed -i 's/\.md/.html/g' {} \;

# Fix specific broken links
find docs/src -type f -name "*.md" -exec sed -i 's/user-guide\/readme\.html/user-guide\/index.html/g' {} \;
find docs/src -type f -name "*.md" -exec sed -i 's/api\/readme\.html/api\/index.html/g' {} \;
find docs/src -type f -name "*.md" -exec sed -i 's/development\/contributing\.html/development\/contributing.html/g' {} \;
find docs/src -type f -name "*.md" -exec sed -i 's/getting-started\/installation\.html/getting-started\/installation.html/g' {} \;

echo "Fixed all internal links!"
echo "Run './scripts/update_docs.sh' to rebuild the documentation with the updated links."
