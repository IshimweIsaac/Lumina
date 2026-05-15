#!/bin/bash
# scripts/upload-binaries.sh
# 
# Usage: ./scripts/upload-binaries.sh <FILE_PATH> <REMOTE_NAME>
# Environment variables required:
# - SUPABASE_URL: e.g., https://woijupkxzzakmkneyxwk.supabase.co
# - SUPABASE_SERVICE_ROLE_KEY: The secret key with storage permissions

set -e

FILE_PATH=$1
REMOTE_NAME=$2

if [ -z "$FILE_PATH" ] || [ -z "$REMOTE_NAME" ]; then
    echo "Usage: $0 <FILE_PATH> <REMOTE_NAME>"
    exit 1
fi

if [ -z "$SUPABASE_URL" ] || [ -z "$SUPABASE_SERVICE_ROLE_KEY" ]; then
    echo "Error: SUPABASE_URL and SUPABASE_SERVICE_ROLE_KEY environment variables must be set."
    exit 1
fi

BUCKET="Lumina"
# Remove 'public/' from URL if it's there, as we use the /object endpoint for uploads
BASE_URL="${SUPABASE_URL}/storage/v1/object/${BUCKET}"

echo "Uploading $FILE_PATH to $BUCKET/$REMOTE_NAME..."

# We use upsert=true so we can overwrite existing binaries for the same version/tag
curl -X POST \
  -H "Authorization: Bearer ${SUPABASE_SERVICE_ROLE_KEY}" \
  -H "Content-Type: application/octet-stream" \
  -H "x-upsert: true" \
  --data-binary @"$FILE_PATH" \
  "${BASE_URL}/${REMOTE_NAME}"

echo "✓ Upload complete."
