#!/bin/sh
# Upload the local release keystore and its credentials as the GitHub
# Actions secrets consumed by tool/setup_android_signing.sh. Run once
# after generating android/app/ghostr-release.jks and android/key.properties,
# and again whenever the key is rotated.
set -eu

repo=${1:-ghostr-social/app}

prop() {
  grep "^$1=" android/key.properties | cut -d= -f2-
}

gh secret set ANDROID_KEYSTORE_BASE64 --repo "$repo" \
  --body "$(base64 -w0 android/app/ghostr-release.jks)"
gh secret set ANDROID_KEYSTORE_PASSWORD --repo "$repo" --body "$(prop storePassword)"
gh secret set ANDROID_KEY_ALIAS --repo "$repo" --body "$(prop keyAlias)"
gh secret set ANDROID_KEY_PASSWORD --repo "$repo" --body "$(prop keyPassword)"

gh secret list --repo "$repo"
