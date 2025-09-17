#!/bin/bash
set -e

# Configuration
MAIN_REPO="rust-ai-ide"
EXAMPLES_REPO="rust-ai-ide-examples"
DOCS_REPO="rust-ai-ide-docs"
GITHUB_USER="your-github-username"  # Replace with your GitHub username
GITHUB_TOKEN="your-github-token"    # Replace with your GitHub personal access token

# Function to create a new GitHub repository
create_github_repo() {
    local repo_name=$1
    local description=$2
    
    echo "Creating repository: $repo_name"
    
    # Make API request to create the repository
    curl -X POST \
        -H "Authorization: token $GITHUB_TOKEN" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/user/repos" \
        -d "{\"name\":\"$repo_name\",\"description\":\"$description\",\"private\":false,\"has_issues\":true,\"has_projects\":true,\"has_wiki\":true}"
    
    # Initialize local directory and push to GitHub
    mkdir -p "../$repo_name"
    cd "../$repo_name"
    git init
    
    # Create basic files
    echo "# $repo_name" > README.md
    echo "$description" >> README.md
    echo "" >> README.md
    echo "## Getting Started" >> README.md
    echo "Add getting started instructions here." >> README.md
    
    # Create .gitignore
    echo "# Rust" > .gitignore
    echo "/target/" >> .gitignore
    echo "/Cargo.lock" >> .gitignore
    echo "" >> .gitignore
    echo "# IDE specific files" >> .gitignore
    echo ".idea/" >> .gitignore
    echo ".vscode/" >> .gitignore
    echo "*.swp" >> .gitignore
    
    # Create LICENSE (MIT)
    echo "MIT License" > LICENSE
    echo "" >> LICENSE
    echo "Copyright (c) $(date +%Y) $GITHUB_USER" >> LICENSE
    echo "" >> LICENSE
    echo "Permission is hereby granted, free of charge, to any person obtaining a copy" >> LICENSE
    echo "of this software and associated documentation files (the \"Software\"), to deal" >> LICENSE
    echo "in the Software without restriction, including without limitation the rights" >> LICENSE
    echo "to use, copy, modify, merge, publish, distribute, sublicense, and/or sell" >> LICENSE
    echo "copies of the Software, and to permit persons to whom the Software is" >> LICENSE
    echo "furnished to do so, subject to the following conditions:" >> LICENSE
    echo "" >> LICENSE
    echo "The above copyright notice and this permission notice shall be included in all" >> LICENSE
    echo "copies or substantial portions of the Software." >> LICENSE
    echo "" >> LICENSE
    echo "THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR" >> LICENSE
    echo "IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY," >> LICENSE
    echo "FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE" >> LICENSE
    echo "AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER" >> LICENSE
    echo "LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM," >> LICENSE
    echo "OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE" >> LICENSE
    echo "SOFTWARE." >> LICENSE
    
    # Create basic directory structure
    mkdir -p src tests
    
    # Create initial commit
    git add .
    git commit -m "Initial commit"
    
    # Add remote and push
    git remote add origin "git@github.com:$GITHUB_USER/$repo_name.git"
    git branch -M main
    git push -u origin main
    
    echo "Created and pushed $repo_name to GitHub"
    cd - > /dev/null
}

# Create main repository
create_github_repo "$MAIN_REPO" "Rust AI IDE - An intelligent development environment built with Rust"

# Create examples repository
create_github_repo "$EXAMPLES_REPO" "Example plugins and extensions for Rust AI IDE"

# Create documentation repository
create_github_repo "$DOCS_REPO" "Documentation for Rust AI IDE"

# Update documentation with new repository URLs
sed -i "s|https://github\.com/rust-ai-ide/rust-ai-ide|https://github.com/$GITHUB_USER/$MAIN_REPO|g" docs/src/**/*.md
sed -i "s|https://github\.com/rust-ai-ide/examples|https://github.com/$GITHUB_USER/$EXAMPLES_REPO|g" docs/src/**/*.md

# Update the main repository's documentation
cd "../$MAIN_REPO"
mkdir -p docs
cp -r "../$MAIN_REPO/docs/"* docs/ 2>/dev/null || true

git add .
git commit -m "docs: update repository URLs"
git push

cd - > /dev/null

echo ""
echo "Successfully created all repositories and updated documentation!"
echo "1. Main repository: https://github.com/$GITHUB_USER/$MAIN_REPO"
echo "2. Examples repository: https://github.com/$GITHUB_USER/$EXAMPLES_REPO"
echo "3. Documentation repository: https://github.com/$GITHUB_USER/$DOCS_REPO"
echo ""
echo "Next steps:"
echo "1. Update the GITHUB_USER and GITHUB_TOKEN variables in this script"
echo "2. Run the script to create the repositories"
echo "3. Enable GitHub Pages for the documentation repository"
echo "4. Update any CI/CD workflows with the new repository URLs"
