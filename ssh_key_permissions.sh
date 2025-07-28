#!/bin/bash

# This script sets recommended secure permissions for the ~/.ssh directory
# and its contents, specifically the authorized_keys file.
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

# 3. Check if the authorized_keys file exists.
if [ -f "$AUTHORIZED_KEYS_FILE" ]; then
    echo "File '$AUTHORIZED_KEYS_FILE' found."
    # 4. Set permissions for the authorized_keys file to 600 (-rw-------).
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

echo "SSH permissions setup complete."
echo "You can verify permissions with: ls -ld ~/.ssh && ls -l ~/.ssh/authorized_keys"
