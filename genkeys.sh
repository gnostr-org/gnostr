#!/bin/bash
EMAIL=${1:-gnostr@gnostr.org}

ssh-keygen -t ed25519 -f gnostr-gnit-key -C "$EMAIL"

# This script sets recommended secure permissions for the ~/.ssh directory
# and its contents, including authorized_keys, private keys, and public keys.
# These permissions are crucial for SSH security and proper functioning.

# Define the SSH directory path
SSH_DIR="$HOME/.ssh"
AUTHORIZED_KEYS_FILE="$SSH_DIR/authorized_keys"

echo "Starting SSH permissions setup..."

# 1. Check if the ~/.ssh directory exists. If not, create it.
if [ ! -d "$SSH_DIR" ]; then
    echo "Directory '$SSH_DIR' does not exist. Creating it..."
    mkdir -p "$SSH_DIR"
    if [ $? -ne 0 ]; then
        echo "Error: Failed to create directory '$SSH_DIR'. Exiting."
        exit 1
    fi
else
    echo "Directory '$SSH_DIR' already exists."
fi

# 2. Set permissions for the ~/.ssh directory to 700 (drwx------).
# This means only the owner can read, write, and execute (access) the directory.
echo "Setting permissions for '$SSH_DIR' to 700..."
chmod 700 "$SSH_DIR"
if [ $? -ne 0 ]; then
    echo "Error: Failed to set permissions for '$SSH_DIR'. Exiting."
    exit 1
else
    echo "Permissions for '$SSH_DIR' set to 700."
fi

# 3. Set permissions for the authorized_keys file.
if [ -f "$AUTHORIZED_KEYS_FILE" ]; then
    echo "File '$AUTHORIZED_KEYS_FILE' found."
    # Set permissions for the authorized_keys file to 600 (-rw-------).
    # This means only the owner can read and write the file.
    echo "Setting permissions for '$AUTHORIZED_KEYS_FILE' to 600..."
    chmod 600 "$AUTHORIZED_KEYS_FILE"
    if [ $? -ne 0 ]; then
        echo "Error: Failed to set permissions for '$AUTHORIZED_KEYS_FILE'. Exiting."
        exit 1
    else
        echo "Permissions for '$AUTHORIZED_KEYS_FILE' set to 600."
    fi
else
    echo "File '$AUTHORIZED_KEYS_FILE' not found. No permissions to set for it."
    echo "Note: If you plan to use SSH keys for authentication, you will need to"
    echo "add your public key to '$AUTHORIZED_KEYS_FILE'."
fi

# 4. Set permissions for private SSH keys.
# Common private key names (without .pub extension)
PRIVATE_KEY_TYPES=("id_rsa" "id_dsa" "id_ecdsa" "id_ed25519" "gnostr-gnit-key")

echo "Checking for and setting permissions for private SSH keys..."
for key_type in "${PRIVATE_KEY_TYPES[@]}"; do
    PRIVATE_KEY_FILE="$SSH_DIR/$key_type"
    if [ -f "$PRIVATE_KEY_FILE" ]; then
        echo "Private key '$PRIVATE_KEY_FILE' found."
        # Set permissions for private keys to 600 (-rw-------).
        # This is critical for security; only the owner should have read/write access.
        echo "Setting permissions for '$PRIVATE_KEY_FILE' to 600..."
        chmod 600 "$PRIVATE_KEY_FILE"
        if [ $? -ne 0 ]; then
            echo "Error: Failed to set permissions for '$PRIVATE_KEY_FILE'. Exiting."
            exit 1
        else
            echo "Permissions for '$PRIVATE_KEY_FILE' set to 600."
        fi
    else
        echo "Private key '$PRIVATE_KEY_FILE' not found."
    fi
done

# 5. Set permissions for public SSH keys.
# Public key files typically end with .pub
PUBLIC_KEY_TYPES=("id_rsa.pub" "id_dsa.pub" "id_ecdsa.pub" "id_ed25519.pub" "gnostr-gnit-key.pub")

echo "Checking for and setting permissions for public SSH keys..."
for key_type in "${PUBLIC_KEY_TYPES[@]}"; do
    PUBLIC_KEY_FILE="$SSH_DIR/$key_type"
    if [ -f "$PUBLIC_KEY_FILE" ]; then
        echo "Public key '$PUBLIC_KEY_FILE' found."
        # Set permissions for public keys to 644 (-rw-r--r--).
        # Public keys can be read by others, but only written by the owner.
        echo "Setting permissions for '$PUBLIC_KEY_FILE' to 644..."
        chmod 644 "$PUBLIC_KEY_FILE"
        if [ $? -ne 0 ]; then
            echo "Error: Failed to set permissions for '$PUBLIC_KEY_FILE'. Exiting."
            exit 1
        else
            echo "Permissions for '$PUBLIC_KEY_FILE' set to 644."
        fi
    else
        echo "Public key '$PUBLIC_KEY_FILE' not found."
    fi
done

echo "SSH permissions setup complete."
echo "You can verify permissions with:"
echo "ls -ld ~/.ssh"
echo "ls -l ~/.ssh/"
