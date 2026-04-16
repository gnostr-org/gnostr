#!/usr/bin/env bash

# Define the alias string with the subshell function logic
# We use a literal string to preserve the complex escaping
LEGIT_FUNCTION='!f() { \
    summary="$1"; \
    prefix="$(gnostr --weeble)/$(gnostr --blockheight)/$(gnostr --wobble)"; \
    gnostr legit -m "$prefix:$summary"; \
}; f'

echo "Checking for 'git legit' alias..."

# Check if the alias already exists
if git config --get alias.legit > /dev/null; then
    echo "✓ Alias 'legit' already exists. Skipping injection."
    echo "Current value: $(git config alias.legit)"
else
    echo "✗ Alias 'legit' not found. Injecting Sovereign metadata wrapper..."
    
    # Inject into global git config
    git config --global alias.legit "$LEGIT_FUNCTION"
    
    if [ $? -eq 0 ]; then
        echo "Successfully injected 'git legit' into .gitconfig"
        echo "Metadata format: [weeble]/[blockheight]/[wobble]:[summary]"
    else
        echo "Error: Failed to inject alias."
        exit 1
    fi
fi

# Show final confirmation
echo -e "\n--- Current legit alias ---"
git config --get alias.legit
