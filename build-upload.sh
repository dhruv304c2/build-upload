#!/bin/bash

# Exit immediately if any command fails
set -e

VERSION="1.0.1"
PROJECT_NAME="Play Doge"
ENV="Staging"

# Check if CLI_VERSION is set
if [ -z "$CLI_VERSION" ]; then
  echo "Error: CLI_VERSION is not set. Please export it before running this script."
  exit 1
fi

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "Script directory: $SCRIPT_DIR"

# Change to the script directory
cd "$SCRIPT_DIR"

# Download the Rust CLI tool
CLI_URL="https://github.com/dhruv304c2/build-upload/releases/download/$CLI_VERSION/build-upload.exe"
OUTPUT_FILE="build-upload.exe"
if curl -L -o "$OUTPUT_FILE" "$CLI_URL"; then
  echo "Downloaded build-upload.exe successfully."
else
  echo "Error: Failed to download build-upload.exe."
  exit 1
fi

# Make the downloaded executable executable (if running on a Unix-like system)
chmod +x "$OUTPUT_FILE"

./"$OUTPUT_FILE" -f "$BUILD_FILE" -n "$FILE_NAME-$SCM_BRANCH-v.$VERSION($UCB_BUILD_NUMBER)" -m "*Builds:*"

if [ $? -eq 0 ]; then
  echo "Build uploaded successfully."
else
  echo "Error: Build upload failed."
  exit 1
fi

if [ -e "./CHANGELOG.md" ]; then
  ./"$OUTPUT_FILE" -f "CHANGELOG.md" -v -m "*[$VERSION][$ENV] $PROJECT_NAME*, Nightly build generated <!everyone>"
  if [ $? -eq 0 ]; then
    echo "Change logs uploaded successfully."
  else
    echo "Error: Change log upload failed."
    exit 1
  fi
else
  echo "No change log file found"
fi
