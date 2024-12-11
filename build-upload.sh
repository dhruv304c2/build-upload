#!/bin/bash

# Exit immediately if any command fails
set -e

VERSION="1.1.0"
PROJECT_NAME="Play Doge"
ENV="Dev"

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

UPLOAD_NAME="$FILE_NAME-$ENV-$SCM_BRANCH-v.$VERSION($UCB_BUILD_NUMBER)"

./"$OUTPUT_FILE" -f "$BUILD_FILE" -n "$UPLOAD_NAME" -m "*Builds:*"

if [ $? -eq 0 ]; then
  echo "Build uploaded successfully."
else
  echo "Error: Build upload failed."
  exit 1
fi

BUNDLETOOL_VERSION="1.13.1"
BUNDLETOOL_URL="https://github.com/google/bundletool/releases/download/$BUNDLETOOL_VERSION/bundletool-all-$BUNDLETOOL_VERSION.jar"

OUTPUT_DIR="bundletool"
TOOL_FILE="$OUTPUT_DIR/bundletool-all-$BUNDLETOOL_VERSION.jar"

mkdir -p "$OUTPUT_DIR"

curl -L "$BUNDLETOOL_URL" -o "$TOOL_FILE"

if [ -f "$TOOL_FILE" ]; then
    echo "BundleTool downloaded successfully to $TOOL_FILE"
else
    echo "Failed to download BundleTool"
    exit 1
fi

OUTPUT_DIR="output_apks"
APK_PATH="$OUTPUT_DIR/$UPLOAD_NAME.apks"

mkdir -p "$OUTPUT_DIR"

java -jar "$TOOL_FILE" build-apks --bundle="$BUILD_FILE" --output="$APK_PATH" --mode=universal

if [ $? -ne 0 ]; then
    echo "Error occurred while extracting APKs from AAB"
    exit 1
else
  ./"$OUTPUT_FILE" -f "$APK_PATH" -n "$UPLOAD_NAME" -m "Note: Use Zarchiver to extract and install Universal.apk"

    if [ $? -eq 0 ]; then
      echo "Build uploaded successfully."
    else
      echo "Error: Build upload failed."
      exit 1
    fi
fi
