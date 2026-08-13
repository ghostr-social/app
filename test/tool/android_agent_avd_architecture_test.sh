#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
fixture=$(mktemp -d)
sdk="$fixture/sdk"
avd_home="$fixture/avd"
name=Ghostr_Agent_API_37.1
package='system-images;android-37.1;google_apis_playstore_ps16k;arm64-v8a'
image_dir="$sdk/system-images/android-37.1/google_apis_playstore_ps16k/arm64-v8a"

cleanup() {
  rm -rf "$fixture"
}
trap cleanup EXIT HUP INT TERM

mkdir -p "$sdk/cmdline-tools/latest/bin" "$image_dir" "$avd_home/$name.avd"
printf 'image.sysdir.1=system-images/android-37.1/google_apis_playstore_ps16k/x86_64/\n' \
  >"$avd_home/$name.avd/config.ini"
printf 'path=%s\n' "$avd_home/$name.avd" >"$avd_home/$name.ini"

cat >"$sdk/cmdline-tools/latest/bin/sdkmanager" <<'SCRIPT'
#!/bin/sh
exit 0
SCRIPT
cat >"$sdk/cmdline-tools/latest/bin/avdmanager" <<'SCRIPT'
#!/bin/sh
set -eu
while [ "$#" -gt 0 ]; do
  case "$1" in
    --name) name=$2; shift 2 ;;
    --package) package=$2; shift 2 ;;
    *) shift ;;
  esac
done
path="$ANDROID_AVD_HOME/$name.avd"
image=$(printf '%s/' "$package" | tr ';' '/')
mkdir -p "$path"
printf 'path=%s\n' "$path" >"$ANDROID_AVD_HOME/$name.ini"
printf 'image.sysdir.1=%s\n' "$image" >"$path/config.ini"
SCRIPT
chmod +x "$sdk/cmdline-tools/latest/bin/sdkmanager" \
  "$sdk/cmdline-tools/latest/bin/avdmanager"

"$root/tool/prepare_android_agent_avd.sh" \
  "$sdk" "$avd_home" "$name" "$package" "$image_dir"

grep -Fq 'image.sysdir.1=system-images/android-37.1/google_apis_playstore_ps16k/arm64-v8a/' \
  "$avd_home/$name.avd/config.ini"
grep -Fq 'disk.dataPartition.size=16G' "$avd_home/$name.avd/config.ini"
backup=$(find "$avd_home" -maxdepth 1 -type d -name "incompatible-$name-*" -print)
test -n "$backup"
test -f "$backup/$name.ini"
test -f "$backup/$name.avd/config.ini"
