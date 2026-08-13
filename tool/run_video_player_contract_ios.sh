#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
name=Ghostr_Player_Contract
started=false
CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_DEV_DEBUG

device_id() {
  xcrun simctl list devices available -j | /usr/bin/ruby -rjson -e '
    devices = JSON.parse(STDIN.read).fetch("devices").values.flatten
    device = devices.find { |item| item["name"] == ARGV[0] }
    puts device["udid"] if device
  ' "$name"
}

runtime_id() {
  xcrun simctl list runtimes -j | /usr/bin/ruby -rjson -e '
    runtimes = JSON.parse(STDIN.read).fetch("runtimes")
    ios = runtimes.select { |item| item["isAvailable"] && item["identifier"].include?("iOS") }
    selected = ios.max_by { |item| item.fetch("version").split(".").map(&:to_i) }
    abort "No available iOS simulator runtime." unless selected
    puts selected.fetch("identifier")
  '
}

device_type_id() {
  xcrun simctl list devicetypes -j | /usr/bin/ruby -rjson -e '
    types = JSON.parse(STDIN.read).fetch("devicetypes")
    selected = types.find { |item| item["name"].start_with?("iPhone") }
    abort "No iPhone simulator device type." unless selected
    puts selected.fetch("identifier")
  '
}

cocoapods_bin_dir() {
  if command -v pod >/dev/null 2>&1; then
    dirname "$(command -v pod)"
    return
  fi
  command -v ruby >/dev/null 2>&1 || return 1
  ruby -rrubygems -e 'Gem.bin_path("cocoapods", "pod"); puts Gem.bindir' 2>/dev/null
}

cleanup() {
  if [ "$started" = true ]; then
    xcrun simctl shutdown "$udid" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT HUP INT TERM

pod_bin_dir=$(cocoapods_bin_dir) || {
  echo 'CocoaPods is required for the locked iOS plugins.' >&2
  exit 1
}
PATH="$pod_bin_dir:$PATH"
export PATH

udid=$(device_id)
if [ -z "$udid" ]; then
  udid=$(xcrun simctl create "$name" "$(device_type_id)" "$(runtime_id)")
fi
state=$(xcrun simctl list devices -j | /usr/bin/ruby -rjson -e '
  devices = JSON.parse(STDIN.read).fetch("devices").values.flatten
  device = devices.find { |item| item["udid"] == ARGV[0] }
  puts device.fetch("state")
' "$udid")
if [ "$state" != Booted ]; then
  xcrun simctl boot "$udid"
  started=true
fi
xcrun simctl bootstatus "$udid" -b

cd "$root"
flutter test --no-pub "$@" -d "$udid"
