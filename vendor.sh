#!/bin/bash
#
# vendor.sh: Automated script for preparing vendor dependencies for a specific Git push.
#
# WARNING: This script is destructive. It recursively deletes the .git folder
# inside each vendor dependency, stages the changes, commits them, and pushes
# immediately on every iteration.

BRANCH_SPEC="$(git branch --show-current):$(git branch --show-current)"
cargo vendor;
for dep in $(/bin/ls vendor);do rm -rf vendor/$dep/.git;git add vendor/$dep; git commit vendor/$dep -m $dep;git push origin $BRANCH_SPEC-vendor;done

# Exit immediately if a command exits with a non-zero status.
set -e

# --- Configuration ---
VENDOR_DIR="vendor"
# Push target uses the format: <remote_name> <source_branch>:<target_branch>
REMOTE_NAME="RandyMcMillan"
BRANCH_SPEC="$(git branch --show-current):$(git branch --show-current)"

# --- Script Execution ---

echo "Starting vendor processing in '$VENDOR_DIR'..."
echo "---"

# Check if the vendor directory exists
if [ ! -d "$VENDOR_DIR" ]; then
    echo "Error: Directory '$VENDOR_DIR' not found. Please create it or run this script from the correct location."
    exit 1
fi

# Loop through all entries in the 'vendor' directory.
# Note: Using /bin/ls as specified in the original request.
for dep in $(/bin/ls "$VENDOR_DIR"); do
    DEP_PATH="$VENDOR_DIR/$dep"

    # Ensure we are processing a directory
    if [ -d "$DEP_PATH" ]; then
        echo "Processing dependency: $dep"

        # 1. Remove existing Git history from the dependency to treat it as source files.
        echo "  - Removing existing .git history from $DEP_PATH"
        rm -rf "$DEP_PATH/.git"

        # 2. Stage the entire dependency directory.
        echo "  - Staging changes for $DEP_PATH"
        git add "$DEP_PATH"

        # 3. Commit the staged changes using the dependency name as the commit message.
        echo "  - Committing changes with message: '$dep'"
        # Commit only the specified path to isolate the changes
        git commit "$DEP_PATH" -m "$dep"

        # 4. Push the committed changes immediately to the specified remote and branch.
        echo "  - Pushing commit to $REMOTE_NAME $BRANCH_SPEC"
        git push "$REMOTE_NAME" "$BRANCH_SPEC"

        echo "---"
    else
        echo "Skipping non-directory item: $dep"
    fi
done

echo "All vendor dependencies processed and pushed successfully."
