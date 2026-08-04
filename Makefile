FLUTTER ?= flutter
ANDROID_ABI ?= arm64-v8a
ANDROID_RELEASE_TARGET ?= android-arm64
ANDROID_DEBUG_TARGET ?= android-x64
ANDROID_DEBUG_ABI ?= x86_64
ANDROID_DEBUG_APK := build/app/outputs/flutter-apk/app-debug.apk
ANDROID_RELEASE_ABI ?= arm64-v8a
ANDROID_RELEASE_APK := build/app/outputs/flutter-apk/app-release.apk
ANDROID_AGENT_SDK ?= $(if $(ANDROID_SDK_ROOT),$(ANDROID_SDK_ROOT),$(ANDROID_HOME))
ANDROID_AGENT_AVD_NAME ?= Ghostr_Agent_API_37.1
ANDROID_AGENT_AVD_PORT ?= 5580
ANDROID_AGENT_AVD_PACKAGE := system-images;android-37.1;google_apis_playstore_ps16k;x86_64
ANDROID_AGENT_AVD_HOME ?= $(if $(ANDROID_AVD_HOME),$(ANDROID_AVD_HOME),$(if $(ANDROID_USER_HOME),$(ANDROID_USER_HOME)/avd,$(HOME)/.android/avd))
ANDROID_AGENT_IMAGE_DIR := $(ANDROID_AGENT_SDK)/system-images/android-37.1/google_apis_playstore_ps16k/x86_64
FLAGS ?=

.PHONY: test-coverage coverage-summary native-check native-test native-coverage rust \
	rust-no-clean gen icons run run-fast run-fast-profile android-debug-apk \
	android-debug-apk-check android-release-apk android-release-apk-check \
	android-agent-avd-create android-agent-avd-run build build-fast install

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

icons:
	inkscape assets/branding/ghostr_icon.svg -w 1024 -h 1024 \
		-o assets/branding/ghostr_icon.png
	inkscape assets/branding/ghostr_icon_foreground.svg -w 1024 -h 1024 \
		-o assets/branding/ghostr_icon_foreground.png
	$(FLUTTER) pub get
	dart run flutter_launcher_icons

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

android-agent-avd-create:
	@test -n "$(ANDROID_AGENT_SDK)" || { echo "Set ANDROID_SDK_ROOT or ANDROID_HOME." >&2; exit 1; }
	@test -x "$(ANDROID_AGENT_SDK)/cmdline-tools/latest/bin/sdkmanager" || { echo "Android command-line tools are missing." >&2; exit 1; }
	@if [ ! -d "$(ANDROID_AGENT_IMAGE_DIR)" ]; then \
		"$(ANDROID_AGENT_SDK)/cmdline-tools/latest/bin/sdkmanager" --install "$(ANDROID_AGENT_AVD_PACKAGE)"; \
	fi
	@if [ ! -f "$(ANDROID_AGENT_AVD_HOME)/$(ANDROID_AGENT_AVD_NAME).ini" ]; then \
		mkdir -p "$(ANDROID_AGENT_AVD_HOME)"; \
		printf 'no\n' | ANDROID_AVD_HOME="$(ANDROID_AGENT_AVD_HOME)" "$(ANDROID_AGENT_SDK)/cmdline-tools/latest/bin/avdmanager" create avd \
			--name "$(ANDROID_AGENT_AVD_NAME)" --package "$(ANDROID_AGENT_AVD_PACKAGE)" \
			--device medium_phone --sdcard 512M; \
	fi
	@avd_path=$$(awk -F= '$$1 == "path" {print $$2; exit}' "$(ANDROID_AGENT_AVD_HOME)/$(ANDROID_AGENT_AVD_NAME).ini"); \
	config="$$avd_path/config.ini"; \
	test -f "$$config" || { echo "AVD config not found: $$config" >&2; exit 1; }; \
	expected_image="system-images/android-37.1/google_apis_playstore_ps16k/x86_64/"; \
	actual_image=$$(awk -F= '$$1 == "image.sysdir.1" {print $$2; exit}' "$$config"); \
	test "$$actual_image" = "$$expected_image" || { echo "Existing AVD uses $$actual_image; it was not modified." >&2; exit 1; }; \
	if grep -q '^disk\.dataPartition\.size=' "$$config"; then \
		sed -i 's/^disk\.dataPartition\.size=.*/disk.dataPartition.size=16G/' "$$config"; \
	else \
		printf 'disk.dataPartition.size=16G\n' >> "$$config"; \
	fi; \
	echo "$(ANDROID_AGENT_AVD_NAME) is ready with 16 GB of durable internal storage."

android-agent-avd-run: android-agent-avd-create
	@exec "$(ANDROID_AGENT_SDK)/emulator/emulator" -avd "$(ANDROID_AGENT_AVD_NAME)" \
		-port "$(ANDROID_AGENT_AVD_PORT)" -netdelay none -netspeed full

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
