#!/bin/bash
set -e

# Update GitHub repository links
github_base="https://github.com/rust-ai-ide/rust-ai-ide"
find docs/src -type f -name "*.md" -exec sed -i "s|https://github\.com/your-org/rust-ai-ide|$github_base|g" {} \;

# Update example plugin links
github_examples="https://github.com/rust-ai-ide/examples"
find docs/src -type f -name "*.md" -exec sed -i "s|https://github\.com/example/code-formatter|$github_examples/tree/main/plugins/code-formatter|g" {} \;
find docs/src -type f -name "*.md" -exec sed -i "s|https://github\.com/example/git-integration|$github_examples/tree/main/plugins/git-integration|g" {} \;
find docs/src -type f -name "*.md" -exec sed -i "s|https://github\.com/example/theme-manager|$github_examples/tree/main/plugins/theme-manager|g" {} \;

# Update documentation links
docs_base="https://rust-ai-ide.github.io/docs"
find docs/src -type f -name "*.md" -exec sed -i "s|https://rust-ai-ide\.example\.com/docs|$docs_base|g" {} \;
find docs/src -type f -name "*.md" -exec sed -i "s|https://rust-ai-ide\.example\.com|$docs_base|g" {} \;

# Update community links
community_forum="https://github.com/rust-ai-ide/rust-ai-ide/discussions"
find docs/src -type f -name "*.md" -exec sed -i "s|https://community\.rust-ai-ide\.example\.com|$community_forum|g" {} \;

# Update issue tracking links
issues="$github_base/issues"
find docs/src -type f -name "*.md" -exec sed -i "s|https://github\.com/your-org/rust-ai-ide/issues|$issues|g" {} \;

# Update release links
releases="$github_base/releases"
find docs/src -type f -name "*.md" -exec sed -i "s|https://github\.com/your-org/rust-ai-ide/releases|$releases|g" {} \;

echo "Updated all external links to point to the correct resources!"
echo "Run './scripts/update_docs.sh' to rebuild the documentation with the updated links."
