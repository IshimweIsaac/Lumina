#!/bin/bash
set -e

echo "Starting massive automated branch merge into main..."

# Ensure we are on main
git checkout main
git pull origin main || true

# Chronological list of all available branches
branches=(
  "ft/1.4-diagnostics"
  "ft/1.4-functions"
  "ft/1.4-go-ffi"
  "ft/1.4-interpolation"
  "ft/1.4-list-types"
  "ft/1.4-modules"
  "ft/1.4-repl-v2"
  "ft/1.4-vscode"
  "v1.4-testing"
  "ft/1.5-external-entities"
  "ft/1.5-lsp"
  "ft/1.5-prev"
  "fix/version-bump-1.5"
  "docs/v1.5-alignment"
  "v1.5-docs-final"
  "v1.5-complete"
  "ft/v1.6-phase1-syntax-ast"
  "ft/v1.6-phase2-semantic-analysis"
  "ft/v1.6-phase3-runtime-engine"
  "ft/v1.6-phase4-lsp-v2"
  "feature/phase4-when-any-all"
  "ft/v1.7-phase1-playground"
  "ft/v1.7-phase2-installer"
  "ft/v1.7-phase3-package-managers"
  "ft/v1.7-phase8-firebase"
  "ft/v1.7-chapter41-website"
  "ft/documents"
)

# Merge sequentially
for branch in "${branches[@]}"; do
  echo "====================================="
  echo "Merging branch: $branch"
  echo "====================================="
  
  # Turn off error exit strictly for the merge command
  set +e
  
  git merge --no-edit origin/$branch -m "Merge $branch into main"
  MERGE_STATUS=$?
  
  if [ $MERGE_STATUS -ne 0 ]; then
    echo "⚠️ Conflict detected merging $branch! Auto-resolving by keeping the incoming features (--theirs)..."
    
    # Identify unmerged files
    unmerged_files=$(git diff --name-only --diff-filter=U)
    
    for file in $unmerged_files; do
        if [ -f "$file" ]; then
            git checkout --theirs "$file"
            git add "$file"
        else
            # File was removed in one branch and modified in another.
            # Usually we remove it if it's missing on incoming branch
            git rm --ignore-unmatch "$file"
        fi
    done
    
    git commit --no-edit -m "Automated conflict resolution: favoring changes from $branch"
  fi
  
  # Re-enable error exit
  set -e
done

echo "====================================="
echo "✅ All branches have been merged into main!"
echo "Pushing main to GitHub..."
git push origin main
echo "Push complete!"
