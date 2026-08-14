#!/bin/sh
set -eu

if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <release-tag> <manifest>" >&2
  exit 64
fi

tag=$1
manifest=$2
version_name=${tag#v}
site_repo=${UPDATE_SITE_REPOSITORY:-ghostr-social/ghostr-social.github.io}
site_workflow=${UPDATE_SITE_WORKFLOW:-sync_stable.yml}
site_ref=${UPDATE_SITE_REF:-main}
site_endpoint=${UPDATE_SITE_ENDPOINT:-https://ghostr-social.github.io/stable.json}
max_attempts=${UPDATE_SITE_MAX_ATTEMPTS:-180}
retry_seconds=${UPDATE_SITE_RETRY_SECONDS:-5}
connect_timeout=${UPDATE_SITE_CONNECT_TIMEOUT_SECONDS:-5}
request_timeout=${UPDATE_SITE_REQUEST_TIMEOUT_SECONDS:-15}
deadline_seconds=${UPDATE_SITE_DEADLINE_SECONDS:-900}
: "${GH_TOKEN:?UPDATE_SITE_TOKEN must be exposed as GH_TOKEN}"

is_unsigned_integer() {
  case "$1" in
    ''|*[!0-9]*) return 1 ;;
    *) return 0 ;;
  esac
}

if ! printf '%s\n' "$tag" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+$' ||
  ! is_unsigned_integer "$max_attempts" || [ "$max_attempts" -eq 0 ] ||
  ! is_unsigned_integer "$retry_seconds" ||
  ! is_unsigned_integer "$connect_timeout" || [ "$connect_timeout" -eq 0 ] ||
  ! is_unsigned_integer "$request_timeout" || [ "$request_timeout" -eq 0 ] ||
  ! is_unsigned_integer "$deadline_seconds" || [ "$deadline_seconds" -eq 0 ]; then
  echo "Update-site publication arguments are invalid." >&2
  exit 65
fi

test -s "$manifest" || {
  echo "Stable release metadata is missing: $manifest" >&2
  exit 66
}
manifest_version=$(jq -er .versionName "$manifest") || {
  echo "Stable release metadata is malformed: $manifest" >&2
  exit 65
}
test "$manifest_version" = "$version_name" || {
  echo "Stable release metadata does not match $tag." >&2
  exit 65
}

observed=$(mktemp "${TMPDIR:-/tmp}/ghostr-stable.XXXXXX")
trap 'rm -f "$observed"' EXIT HUP INT TERM
deadline_at=$(( $(date +%s) + deadline_seconds ))

gh workflow run "$site_workflow" --repo "$site_repo" --ref "$site_ref"

attempt=1
while [ "$attempt" -le "$max_attempts" ] &&
  [ "$(date +%s)" -lt "$deadline_at" ]; do
  if curl --fail --silent --show-error --location --max-redirs 3 \
    --connect-timeout "$connect_timeout" --max-time "$request_timeout" \
    --header 'Cache-Control: no-cache' "$site_endpoint" > "$observed" &&
    cmp -s "$manifest" "$observed"; then
    echo "Published Ghostr $version_name to $site_endpoint"
    exit 0
  fi
  if [ "$attempt" -lt "$max_attempts" ] &&
    [ "$(date +%s)" -lt "$deadline_at" ]; then
    sleep "$retry_seconds"
  fi
  attempt=$((attempt + 1))
done

echo "The app-facing update catalog did not publish Ghostr $version_name." >&2
exit 69
