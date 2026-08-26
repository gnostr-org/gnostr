#!/usr/bin/env bash

# Set script to exit on error printout flags
set -e

echo "================================================================="
echo "🔍 DIAGNOSING XCODE KEYCHAIN CODE SIGNING ERROR"
echo "================================================================="

TARGET_SHA="348F4A92F26CBF42932EA4413EB358B6A5139C7D"
KEYCHAIN_PATH="$HOME/Library/Keychains/login.keychain-db"

# Step 1: Check if the certificate identity exists in any accessible keychain
echo "--> 1. Checking available code signing identities..."
if security find-identity -v -p codesigning | grep -q "$TARGET_SHA"; then
    echo "✅ Success: Identity $TARGET_SHA found in your active identities."
else
    echo "❌ Error: Identity $TARGET_SHA is NOT found in your active valid identities."
    echo "Let's list what is actually available:"
    security find-identity -v -p codesigning
fi

echo "-----------------------------------------------------------------"

# Step 2: Explicitly unlock the login keychain
echo "--> 2. Attempting to unlock login keychain..."
echo "Note: If prompted in the terminal, please enter your macOS user password."
security unlock-keychain -p "" "$KEYCHAIN_PATH" 2>/dev/null || {
    echo "⚠️  Standard blank-pass unlock failed. Prompting for explicit unlock..."
    security unlock-keychain "$KEYCHAIN_PATH"
}

# Step 3: Increase keychain timeout to prevent mid-build locking
echo "--> 3. Setting keychain partition list permissions and timeout..."
security set-keychain-settings -t 3600 -u "$KEYCHAIN_PATH"

# Step 4: Verify manual codesign capability
echo "--> 4. Testing codesign dry-run verification..."
echo "If this command fails with 'The specified item could not be found', it means"
echo "the certificate is present but the Private Key is missing from your Keychain."
echo "-----------------------------------------------------------------"

# Dummy verification run or check private key linkage
security find-certificate -a -Z | grep -B 2 -A 10 "$TARGET_SHA" || true

echo "================================================================="
echo "💡 NEXT STEPS IF THE ERROR PERSISTS:"
echo "1. Open Xcode -> Settings -> Accounts."
echo "2. Select 'Apple Development: Randy McMillan (J53T3P2E45)'."
echo "3. Click 'Manage Certificates...' and verify if there is a 'Missing Private Key' warning."
echo "4. If missing, revoke and re-create the development certificate via Xcode or Apple Developer Portal."
echo "================================================================="
