#!/bin/sh
# Restore the stable Android release keystore from CI secrets so every
# published APK is signed with the same certificate. Without it, each CI
# runner generates a fresh debug keystore and installed apps reject the
# update with STATUS_FAILURE_CONFLICT (incompatible certificates).
set -eu

if [ -z "${ANDROID_KEYSTORE_BASE64:-}" ]; then
  if [ "${GITHUB_EVENT_NAME:-}" = "pull_request" ]; then
    echo "Signing secrets unavailable (pull request build); using debug signing." >&2
    exit 0
  fi
  echo "::error::ANDROID_KEYSTORE_BASE64 secret is not set; refusing to build a throwaway-signed release. Run tool/upload_android_signing_secrets.sh." >&2
  exit 1
fi

printf '%s' "$ANDROID_KEYSTORE_BASE64" | base64 -d > android/app/ghostr-release.jks

cat > android/key.properties <<EOF
storePassword=$ANDROID_KEYSTORE_PASSWORD
keyPassword=$ANDROID_KEY_PASSWORD
keyAlias=$ANDROID_KEY_ALIAS
storeFile=ghostr-release.jks
EOF
