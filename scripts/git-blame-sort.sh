#!/bin/bash

# Check if a filename was provided
if [ -z "$1" ]; then
    echo "Usage: $0 <filename>"
    exit 1
fi

FILE=$1

# Check if the file exists
if [ ! -f "$FILE" ]; then
    echo "Error: File '$FILE' not found."
    exit 1
fi

echo "Sorting git blame for: $FILE (Oldest to Newest)"
echo "------------------------------------------------"

# 1. git blame --line-porcelain: Get raw machine-readable output
# 2. awk: Extract timestamp and short hash
# 3. sort -n: Sort numerically by timestamp
# 4. while loop: Convert epoch to human-readable using macOS/BSD 'date -r'
git blame --line-porcelain "$FILE" | \
awk '/^([0-9a-f]{40})/ {h=substr($1,1,7)} /^author-time/ {print $2, h}' | \
sort -n | \
while read -r timestamp hash; do
    # -r is for macOS/BSD. Use -d @"$timestamp" for GNU/Linux
    readable_date=$(date -r "$timestamp" "+%Y-%m-%d %H:%M:%S")
    printf "[%s] Commit: %s\n" "$readable_date" "$hash"
done
