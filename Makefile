FLUTTER ?= flutter
ANDROID_ABI ?= arm64-v8a
ANDROID_RELEASE_TARGET ?= android-arm64
ANDROID_DEBUG_TARGET ?= android-x64
ANDROID_DEBUG_ABI ?= x86_64
ANDROID_DEBUG_APK := build/app/outputs/flutter-apk/app-debug.apk
ANDROID_RELEASE_ABI ?= arm64-v8a
ANDROID_RELEASE_APK := build/app/outputs/flutter-apk/app-release.apk
FLAGS ?=

.PHONY: test-coverage coverage-summary native-check native-test native-coverage rust \
	rust-no-clean gen run run-fast run-fast-profile android-debug-apk \
	android-debug-apk-check android-release-apk android-release-apk-check \
	build build-fast install

test-coverage:
	$(FLUTTER) test --coverage

coverage-summary:
	@awk 'BEGIN{FS=":"; include=1} /^SF:/{include=($$0 !~ /lib\/src\/rust\//)} include && /^DA:/{split($$2,a,","); if (a[2] > 0) hit++; total++} END {if (total == 0) {print "No coverage data"; exit 1} printf("Line coverage: %.2f%% (%d/%d)\n", (hit/total)*100, hit, total)}' coverage/lcov.info
	@sh tool/check_dart_coverage_sources.sh coverage/lcov.info lib tool/dart_coverage_exclusions.txt
	@awk -f tool/check_dart_coverage.awk coverage/lcov.info

native-check:
	cd rust && cargo check

native-test:
	cd rust && cargo test

native-coverage:
	cd rust && cargo llvm-cov --ignore-filename-regex 'frb_generated.rs' --lcov --output-path target/native-coverage.lcov --fail-under-lines 95
	awk -f tool/check_native_coverage.awk rust/target/native-coverage.lcov

rust:
	cd rust && cargo clean
	$(MAKE) rust-no-clean

rust-no-clean:
	cd rust && cargo ndk -t "$(ANDROID_ABI)" build

gen:
	flutter_rust_bridge_codegen generate

run:
	$(FLUTTER) run $(FLAGS)

run-fast:
	$(FLUTTER) run $(FLAGS)

run-fast-profile:
	$(FLUTTER) run --profile $(FLAGS)

android-debug-apk:
	$(FLUTTER) build apk --debug --target-platform "$(ANDROID_DEBUG_TARGET)"

android-debug-apk-check: android-debug-apk
	@sh tool/check_android_apk_abi.sh "$(ANDROID_DEBUG_APK)" "$(ANDROID_DEBUG_ABI)"

android-release-apk:
	$(FLUTTER) build apk --release --target-platform "$(ANDROID_RELEASE_TARGET)" --config-only
	$(FLUTTER) build apk --release --target-platform "$(ANDROID_RELEASE_TARGET)"

android-release-apk-check: android-release-apk
	@sh tool/check_android_apk_abi.sh "$(ANDROID_RELEASE_APK)" "$(ANDROID_RELEASE_ABI)"
	@sh tool/check_android_release_apk.sh "$(ANDROID_RELEASE_APK)"

build:
	$(MAKE) android-release-apk
	$(MAKE) install

build-fast:
	$(MAKE) android-release-apk
	$(MAKE) install

APK_PATH := build/app/outputs/flutter-apk
VERSION := app-release-$(shell date +'%Y-%m-%d-%H-%M').apk

install:
	mv "$(APK_PATH)/app-release.apk" "$(APK_PATH)/$(VERSION)"
	@echo "$(VERSION)"
	adb push "$(APK_PATH)/$(VERSION)" /sdcard/
	adb install -r "$(APK_PATH)/$(VERSION)"
