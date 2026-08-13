#!/bin/sh
set -eu

if [ "$#" -ne 5 ]; then
  echo 'Usage: prepare_android_agent_avd.sh SDK AVD_HOME NAME PACKAGE IMAGE_DIR' >&2
  exit 64
fi

sdk=$1
avd_home=$2
name=$3
package=$4
image_dir=$5
sdkmanager="$sdk/cmdline-tools/latest/bin/sdkmanager"
avdmanager="$sdk/cmdline-tools/latest/bin/avdmanager"
ini="$avd_home/$name.ini"
expected_image=$(printf '%s/' "$package" | tr ';' '/')

archive_incompatible_avd() {
  avd_path=$1
  case "$avd_path" in
    "$avd_home"/*) ;;
    *) echo "Refusing to move AVD outside $avd_home: $avd_path" >&2; exit 1 ;;
  esac
  test -d "$avd_path" || { echo "AVD path not found: $avd_path" >&2; exit 1; }
  backup="$avd_home/incompatible-$name-$(date +%Y%m%d%H%M%S)-$$"
  mkdir -p "$backup"
  mv "$avd_path" "$backup/$name.avd"
  mv "$ini" "$backup/$name.ini"
  echo "Archived incompatible $name at $backup."
}

set_data_partition_size() {
  config=$1
  temporary=$(mktemp "$config.XXXXXX")
  awk '
    BEGIN { replaced = 0 }
    /^disk\.dataPartition\.size=/ && !replaced {
      print "disk.dataPartition.size=16G"; replaced = 1; next
    }
    /^disk\.dataPartition\.size=/ { next }
    { print }
    END { if (!replaced) print "disk.dataPartition.size=16G" }
  ' "$config" >"$temporary"
  mv "$temporary" "$config"
}

test -x "$sdkmanager" || { echo "Android sdkmanager is missing: $sdkmanager" >&2; exit 1; }
test -x "$avdmanager" || { echo "Android avdmanager is missing: $avdmanager" >&2; exit 1; }
mkdir -p "$avd_home"
if [ ! -d "$image_dir" ]; then
  "$sdkmanager" --install "$package"
fi

if [ -f "$ini" ]; then
  avd_path=$(awk -F= '$1 == "path" {print $2; exit}' "$ini")
  config="$avd_path/config.ini"
  actual_image=$(awk -F= '$1 == "image.sysdir.1" {print $2; exit}' "$config" 2>/dev/null || true)
  if [ "$actual_image" != "$expected_image" ]; then
    archive_incompatible_avd "$avd_path"
  fi
fi

if [ ! -f "$ini" ]; then
  printf 'no\n' | ANDROID_AVD_HOME="$avd_home" "$avdmanager" create avd \
    --name "$name" --package "$package" --device medium_phone --sdcard 512M
fi

avd_path=$(awk -F= '$1 == "path" {print $2; exit}' "$ini")
config="$avd_path/config.ini"
test -f "$config" || { echo "AVD config not found: $config" >&2; exit 1; }
actual_image=$(awk -F= '$1 == "image.sysdir.1" {print $2; exit}' "$config")
test "$actual_image" = "$expected_image" || {
  echo "AVD image mismatch: expected $expected_image, found $actual_image" >&2
  exit 1
}
set_data_partition_size "$config"
echo "$name is ready with 16 GB of durable internal storage."
