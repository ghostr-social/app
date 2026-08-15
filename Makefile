FLUTTER ?= flutter
ADB ?= adb
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
ANDROID_AGENT_HOST_ARCH ?= $(shell uname -m)
ANDROID_AGENT_SYSTEM_IMAGE_ABI ?= $(if $(filter arm64 aarch64,$(ANDROID_AGENT_HOST_ARCH)),arm64-v8a,x86_64)
ANDROID_AGENT_AVD_PACKAGE := system-images;android-37.1;google_apis_playstore_ps16k;$(ANDROID_AGENT_SYSTEM_IMAGE_ABI)
ANDROID_AGENT_AVD_HOME ?= $(if $(ANDROID_AVD_HOME),$(ANDROID_AVD_HOME),$(if $(ANDROID_USER_HOME),$(ANDROID_USER_HOME)/avd,$(HOME)/.android/avd))
ANDROID_AGENT_IMAGE_DIR := $(ANDROID_AGENT_SDK)/system-images/android-37.1/google_apis_playstore_ps16k/$(ANDROID_AGENT_SYSTEM_IMAGE_ABI)
ANDROID_GRADLE_JAVA_HOME ?= $(shell if [ -x /usr/libexec/java_home ]; then /usr/libexec/java_home -v 21; fi)
ANDROID_GRADLE_JAVA_OPTION := $(if $(ANDROID_GRADLE_JAVA_HOME),-Dorg.gradle.java.home=$(ANDROID_GRADLE_JAVA_HOME),)
VIDEO_ANDROID_EMULATOR_SERIAL ?= emulator-5580
ANDROID_PHYSICAL_SERIAL ?=
VIDEO_IMPAIRMENT_SCENARIOS := bandwidth_drop packet_loss high_rtt rapid_swipes \
	storage_pressure source_failure protected_transitions
VIDEO_BROWSER_SCENARIOS := adaptive_plans $(VIDEO_IMPAIRMENT_SCENARIOS)
VIDEO_ANDROID_INTEGRATION_TESTS := \
	integration_test/bandwidth_drop_video_test.dart \
	integration_test/packet_loss_video_test.dart \
	integration_test/high_rtt_video_test.dart \
	integration_test/rapid_swipes_video_test.dart \
	integration_test/held_response_video_test.dart \
	integration_test/manifest_retry_video_test.dart
VIDEO_PLAYER_CONTRACT_TESTS := \
	integration_test/video_player_lifecycle_contract_test.dart \
	integration_test/video_player_error_contract_test.dart \
	integration_test/video_player_stall_contract_test.dart \
	integration_test/video_player_preparation_contract_test.dart \
	integration_test/video_player_prepared_generation_contract_test.dart \
	integration_test/video_player_adapter_identity_contract_test.dart
VIDEO_PROGRESSIVE_ANDROID_TESTS := \
	integration_test/progressive_delivery_video_test.dart
VIDEO_PROGRESSIVE_FLUTTER_TESTS := \
	test/core/media/remote_playback_delivery_id_test.dart \
	test/media/ffi_playback_telemetry_cross_port_generation_test.dart \
	test/media/ffi_playback_telemetry_deactivation_collapse_test.dart \
	test/media/ffi_playback_telemetry_delivery_identity_test.dart \
	test/media/ffi_playback_telemetry_late_deactivation_test.dart \
	test/media/ffi_playback_telemetry_terminal_synthesis_test.dart \
	test/media/ffi_feed_focus_cross_port_generation_test.dart \
	test/media/ffi_video_gateway_device_integration_scope_test.dart \
	test/media/progressive_device_fixture_test.dart \
	test/media/progressive_device_origin_cancellation_accounting_test.dart \
	test/media/progressive_device_origin_test.dart \
	test/media/progressive_device_resources_test.dart \
	test/media/progressive_device_wait_deadline_test.dart \
	test/video_catalog/feed_load_more_appends_test.dart \
	test/video_catalog/feed_backfill_dry_cursor_test.dart \
	test/video_catalog/feed_backfill_retry_rechecks_buffer_test.dart \
	test/video_catalog/feed_backfill_stationary_cursor_test.dart \
	test/video_catalog/feed_refresh_focus_republish_test.dart \
	test/video_catalog/feed_focus_inactive_write_test.dart \
	test/video_catalog/feed_focus_atomic_reactivation_test.dart \
	test/video_catalog/feed_focus_lease_disposal_test.dart \
	test/video_catalog/feed_focus_lease_contract_test.dart \
	test/video_catalog/feed_focus_sink_contract_test.dart \
	test/video_catalog/feed_roster_resync_media_dedup_test.dart \
	test/app/home_feed_focus_route_return_test.dart \
	test/app/nested_feed_focus_return_test.dart \
	test/app/home_feed_focus_tab_test.dart
FLAGS ?=
WEB_DEBUG_CACHE_DIR ?= $(CURDIR)/rust/target/video-debug-cache
WEB_DEBUG_RUST_DIR ?= $(CURDIR)/rust
WEB_DEBUG_STATE_DIR ?= $(CURDIR)/rust/target/video-debug-web
HAWK_REPOSITORY := https://github.com/gu1p/hawk
HAWK_REVISION := 98efa9f7590d12672ece0527e4a908788792a997
HAWK_REVISION_SHORT := 98efa9f

.PHONY: test-coverage coverage-summary native-check native-test native-coverage web \
	native-dead-code-install native-dead-code native-dead-code-contract-test \
	web-contract-test video-user-e2e video-user-e2e-contract-test \
	video-demo \
	video-user-e2e-prerequisite-check video-user-e2e-impairments \
	video-delivery-target-contract-test video-android-emulator-tests \
	video-android-physical-tests video-player-contract-target-test \
	video-player-contract video-player-contract-android video-player-contract-ios \
	video-progressive-suite-contract-test video-progressive-suite \
	video-progressive-android \
	native-coverage-contract-test rust rust-no-clean gen icons run run-fast \
	run-fast-profile android-unit-tests android-debug-apk android-debug-apk-check \
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

native-dead-code-install: ## Install the Rust toolchain and Hawk dead-code analyzer.
	@if ! rustup component list --toolchain 1.97.1 --installed 2>/dev/null | grep -q '^rustc-dev-'; then \
		rustup toolchain install 1.97.1 --component rustc-dev; \
	fi
	@if ! cargo +1.97.1 install --list 2>/dev/null | grep -q '$(HAWK_REVISION_SHORT)'; then \
		RUSTC_BOOTSTRAP=1 cargo +1.97.1 install --locked --force \
			--git "$(HAWK_REPOSITORY)" --rev "$(HAWK_REVISION)" cargo-hawk; \
	fi

native-dead-code: ## Find Rust declarations reachable only from tests.
	cd rust && cargo +1.97.1 hawk check --only test-only -D hawk::test_only

native-dead-code-contract-test: ## Verify dead-code checks do not inspect user-scoped tooling.
	sh test/tool/native_dead_code_target_contract_test.sh

native-test: web-contract-test ## Run Rust tests.
	cd rust && cargo test -p ghostr-gateway --no-default-features --test debug_web_exclusion_test
	cd rust && cargo test --workspace --all-features

web-contract-test: ## Verify that the web tool is Rust-only.
	sh test/tool/web_target_contract_test.sh
	sh test/tool/web_lifecycle_contract_test.sh
	sh test/tool/web_untrusted_owner_contract_test.sh

video-user-e2e-contract-test: ## Test the deterministic local video E2E harness.
	node --test test/tool/video_user_e2e/*_test.mjs

video-user-e2e-prerequisite-check: ## Verify the configured browser executable without launching it.
	node tool/video_user_e2e/main.mjs --check-prerequisites

video-user-e2e: video-user-e2e-contract-test ## Run the deterministic local browser journey.
	node tool/video_user_e2e/main.mjs

video-demo: video-user-e2e-contract-test ## Run an observable eight-video WARP retrieval demo.
	node tool/video_user_e2e/demo_main.mjs

video-user-e2e-impairments: video-user-e2e-contract-test ## Run every deterministic browser impairment.
	@set -e; for scenario in $(VIDEO_BROWSER_SCENARIOS); do \
		node tool/video_user_e2e/main.mjs --scenario="$$scenario"; \
	done

video-delivery-target-contract-test: ## Verify stable browser and Android video test targets.
	sh test/tool/video_browser_impairment_target_contract_test.sh
	sh test/tool/video_android_emulator_target_contract_test.sh
	sh test/tool/video_android_physical_target_contract_test.sh

video-android-emulator-tests: ## Run the device video playback matrix on emulator-5580.
	$(FLUTTER) test $(VIDEO_ANDROID_INTEGRATION_TESTS) -d "$(VIDEO_ANDROID_EMULATOR_SERIAL)"

video-player-contract-target-test: ## Verify automatic player contract targets.
	sh test/tool/video_player_contract_target_test.sh

video-player-contract: video-player-contract-android video-player-contract-ios ## Run both locked player contracts.

video-player-contract-android: video-player-contract-target-test ## Run the player contract on the repository AVD.
	tool/run_video_player_contract_android.sh $(VIDEO_PLAYER_CONTRACT_TESTS)

video-player-contract-ios: video-player-contract-target-test ## Run the player contract on an automatic iOS simulator.
	tool/run_video_player_contract_ios.sh $(VIDEO_PLAYER_CONTRACT_TESTS)

video-progressive-suite-contract-test: ## Verify the progressive suite and QoE contracts.
	sh test/tool/video_progressive_suite_target_test.sh
	cd rust && cargo test -p ghostr-delivery \
		--test delivery_next_reserve_evidence_test --all-features

video-progressive-suite: video-progressive-suite-contract-test ## Run the repaired progressive-path suite.
	cd rust && cargo test -p ghostr-engine --all-features
	cd rust && cargo test -p ghostr-delivery --all-features
	cd rust && cargo test -p ghostr-gateway --all-features
	cd rust && cargo test -p rust_lib_ghostr --all-features
	$(FLUTTER) test --no-pub $(VIDEO_PROGRESSIVE_FLUTTER_TESTS)

video-progressive-android: video-progressive-suite-contract-test ## Run progressive playback on the repository AVD.
	tool/run_video_player_contract_android.sh $(VIDEO_PROGRESSIVE_ANDROID_TESTS)

video-android-physical-tests: ## Run the device video playback matrix on physical Android.
	@test -n "$(ANDROID_PHYSICAL_SERIAL)" || { echo "Set ANDROID_PHYSICAL_SERIAL to an attached device serial." >&2; exit 1; }
	@case "$(ANDROID_PHYSICAL_SERIAL)" in emulator-*) echo "ANDROID_PHYSICAL_SERIAL must identify physical hardware." >&2; exit 1;; esac
	@state=$$($(ADB) -s "$(ANDROID_PHYSICAL_SERIAL)" get-state 2>/dev/null || true); \
		test "$$state" = device || { echo "Android device $(ANDROID_PHYSICAL_SERIAL) is not ready." >&2; exit 1; }; \
		qemu=$$($(ADB) -s "$(ANDROID_PHYSICAL_SERIAL)" shell getprop ro.kernel.qemu | tr -d '\r'); \
		test "$$qemu" != 1 || { echo "ANDROID_PHYSICAL_SERIAL must identify physical hardware." >&2; exit 1; }
	$(FLUTTER) test $(VIDEO_ANDROID_INTEGRATION_TESTS) -d "$(ANDROID_PHYSICAL_SERIAL)"

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

android-unit-tests: ## Run host-side Android bridge and share-receiver tests.
	$(FLUTTER) pub get
	cd android && ./gradlew $(ANDROID_GRADLE_JAVA_OPTION) \
		:app:incomingVideoShareCoverageCheck

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
	@tool/prepare_android_agent_avd.sh \
		"$(ANDROID_AGENT_SDK)" "$(ANDROID_AGENT_AVD_HOME)" \
		"$(ANDROID_AGENT_AVD_NAME)" "$(ANDROID_AGENT_AVD_PACKAGE)" \
		"$(ANDROID_AGENT_IMAGE_DIR)"

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
