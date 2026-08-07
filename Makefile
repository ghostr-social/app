FLUTTER ?= flutter
FLUTTER_TEST_CONCURRENCY ?= 4
FLUTTER_TEST_OPEN_FILES ?= 4096
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
WEB_DEBUG_CACHE_DIR ?= $(CURDIR)/rust/target/video-debug-cache
WEB_DEBUG_RUST_DIR ?= $(CURDIR)/rust
WEB_DEBUG_STATE_DIR ?= $(CURDIR)/rust/target/video-debug-web

.PHONY: test-coverage coverage-summary native-check native-test native-coverage web \
	web-contract-test \
	native-coverage-contract-test rust rust-no-clean gen icons run run-fast \
	run-fast-profile android-debug-apk android-debug-apk-check \
	android-release-apk android-release-apk-check android-agent-avd-create \
	android-agent-avd-run build build-fast install help

test-coverage: ## Run Flutter tests and collect Dart coverage.
	@ulimit -n "$(FLUTTER_TEST_OPEN_FILES)"; \
	exec $(FLUTTER) test --coverage --concurrency="$(FLUTTER_TEST_CONCURRENCY)"

coverage-summary: ## Report and enforce Dart coverage requirements.
	@awk 'BEGIN{FS=":"; include=1} /^SF:/{include=($$0 !~ /lib\/src\/rust\//)} include && /^DA:/{split($$2,a,","); if (a[2] > 0) hit++; total++} END {if (total == 0) {print "No coverage data"; exit 1} printf("Line coverage: %.2f%% (%d/%d)\n", (hit/total)*100, hit, total)}' coverage/lcov.info
	@sh tool/check_dart_coverage_sources.sh coverage/lcov.info lib tool/dart_coverage_exclusions.txt
	@awk -f tool/check_dart_coverage.awk coverage/lcov.info

native-check: ## Check the Rust package.
	cd rust && cargo clippy --workspace --all-targets --all-features -- -D warnings

native-test: web-contract-test ## Run Rust tests.
	cd rust && cargo test --no-default-features --test debug_web_exclusion_test
	cd rust && cargo test --workspace --all-features

web-contract-test: ## Verify that the web tool is Rust-only.
	sh test/tool/web_target_contract_test.sh
	sh test/tool/web_lifecycle_contract_test.sh
	sh test/tool/web_untrusted_owner_contract_test.sh

native-coverage-contract-test: ## Test the per-file native coverage contract.
	sh test/tool/native_coverage_contract_test.sh

native-coverage: native-coverage-contract-test ## Run Rust tests and enforce native coverage.
	cd rust && cargo llvm-cov --workspace --all-features --ignore-filename-regex 'frb_generated.rs' --lcov --output-path target/native-coverage.lcov --fail-under-lines 95
	awk -f tool/check_native_coverage.awk rust/target/native-coverage.lcov

web: ## Run the standalone Rust video debugging dashboard.
	@sh "$(CURDIR)/tool/run_video_debug_web.sh" \
		"$(WEB_DEBUG_CACHE_DIR)" "$(WEB_DEBUG_STATE_DIR)" "$(WEB_DEBUG_RUST_DIR)"

rust: ## Clean and build the Rust Android library.
	cd rust && cargo clean
	$(MAKE) rust-no-clean

rust-no-clean: ## Build the Rust Android library without cleaning.
	cd rust && cargo ndk -t "$(ANDROID_ABI)" build -p rust_lib_ghostr

gen: ## Generate the Flutter-Rust bridge bindings.
	flutter_rust_bridge_codegen generate

icons: ## Generate app icon assets.
	inkscape assets/branding/ghostr_icon.svg -w 1024 -h 1024 \
		-o assets/branding/ghostr_icon.png
	inkscape assets/branding/ghostr_icon_foreground.svg -w 1024 -h 1024 \
		-o assets/branding/ghostr_icon_foreground.png
	$(FLUTTER) pub get
	dart run flutter_launcher_icons

run: ## Run the Flutter app.
	$(FLUTTER) run $(FLAGS)

run-fast: ## Run the Flutter app.
	$(FLUTTER) run $(FLAGS)

run-fast-profile: ## Run the Flutter app in profile mode.
	$(FLUTTER) run --profile $(FLAGS)

android-debug-apk: ## Build the Android debug APK.
	$(FLUTTER) build apk --debug --target-platform "$(ANDROID_DEBUG_TARGET)"

android-debug-apk-check: android-debug-apk ## Build the Android debug APK and verify its ABI.
	@sh tool/check_android_apk_abi.sh "$(ANDROID_DEBUG_APK)" "$(ANDROID_DEBUG_ABI)"

android-release-apk: ## Build the Android release APK.
	$(FLUTTER) build apk --release --target-platform "$(ANDROID_RELEASE_TARGET)" --config-only
	$(FLUTTER) build apk --release --target-platform "$(ANDROID_RELEASE_TARGET)"

android-release-apk-check: android-release-apk ## Build and validate the Android release APK.
	@sh tool/check_android_apk_abi.sh "$(ANDROID_RELEASE_APK)" "$(ANDROID_RELEASE_ABI)"
	@sh tool/check_android_release_apk.sh "$(ANDROID_RELEASE_APK)"

android-agent-avd-create: ## Install and create the dedicated Android agent AVD.
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

android-agent-avd-run: android-agent-avd-create ## Start the dedicated Android agent AVD on emulator-5580.
	@exec "$(ANDROID_AGENT_SDK)/emulator/emulator" -avd "$(ANDROID_AGENT_AVD_NAME)" \
		-port "$(ANDROID_AGENT_AVD_PORT)" -netdelay none -netspeed full

build: ## Build and install the Android release APK.
	$(MAKE) android-release-apk
	$(MAKE) install

build-fast: ## Build and install the Android release APK.
	$(MAKE) android-release-apk
	$(MAKE) install

APK_PATH := build/app/outputs/flutter-apk
VERSION := app-release-$(shell date +'%Y-%m-%d-%H-%M').apk

install: ## Rename, push, and install the Android release APK.
	mv "$(APK_PATH)/app-release.apk" "$(APK_PATH)/$(VERSION)"
	@echo "$(VERSION)"
	adb push "$(APK_PATH)/$(VERSION)" /sdcard/
	adb install -r "$(APK_PATH)/$(VERSION)"

help: ## List all commands and their descriptions.
	@printf 'Available commands:\n'
	@awk 'BEGIN {FS = ":.*## "} /^[[:alnum:]_.-]+:.*## / {printf "  %-31s %s\n", $$1, $$2}' $(MAKEFILE_LIST) | sort
