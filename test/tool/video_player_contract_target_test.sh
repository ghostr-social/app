#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
makefile="$root/Makefile"
android="$root/tool/run_video_player_contract_android.sh"
android_prepare="$root/tool/prepare_android_agent_avd.sh"
ios="$root/tool/run_video_player_contract_ios.sh"

grep -Fq 'video-player-contract-android:' "$makefile"
grep -Fq 'video-player-contract-ios:' "$makefile"
grep -Fq 'VIDEO_PLAYER_CONTRACT_TESTS :=' "$makefile"

for test_name in lifecycle error stall preparation prepared_generation adapter_identity
do
  grep -Fq "integration_test/video_player_${test_name}_contract_test.dart" \
    "$makefile"
done

test -x "$android"
grep -Fq 'make android-agent-avd-create' "$android"
grep -Fq 'make android-agent-avd-run' "$android"
grep -Fq 'emulator-5580' "$android"
grep -Fq 'sys.boot_completed' "$android"
grep -Fq 'CARGO_PROFILE_DEV_DEBUG=0' "$android"
grep -Fq 'ANDROID_AGENT_SYSTEM_IMAGE_ABI' "$makefile"
grep -Fq 'tool/prepare_android_agent_avd.sh' "$makefile"
test -x "$android_prepare"
grep -Fq 'incompatible-' "$android_prepare"
! grep -Eq -- '--force|-wipe-data|rm -rf' "$android_prepare"

test -x "$ios"
grep -Fq 'simctl list' "$ios"
grep -Fq 'simctl create' "$ios"
grep -Fq 'simctl bootstatus' "$ios"
grep -Fq 'Ghostr_Player_Contract' "$ios"
grep -Fq 'Gem.bin_path("cocoapods", "pod")' "$ios"
grep -Fq 'CARGO_PROFILE_DEV_DEBUG=0' "$ios"
test -f "$root/ios/Podfile"
grep -Fq '#include? "Pods/Target Support Files/Pods-Runner/Pods-Runner.debug.xcconfig"' \
  "$root/ios/Flutter/Debug.xcconfig"
grep -Fq '#include? "Pods/Target Support Files/Pods-Runner/Pods-Runner.release.xcconfig"' \
  "$root/ios/Flutter/Release.xcconfig"
/usr/bin/ruby -ryaml -e '
  config = YAML.safe_load(File.read(ARGV[0])).dig("flutter", "config")
  abort "Swift Package Manager must be disabled for CocoaPods-only plugins" unless
    config == {"enable-swift-package-manager" => false}
' "$root/pubspec.yaml"
grep -Fq 'FlutterImplicitEngineDelegate' "$root/ios/Runner/AppDelegate.swift"
grep -Fq '<key>UIApplicationSceneManifest</key>' "$root/ios/Runner/Info.plist"
