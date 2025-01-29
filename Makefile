.PHONY: rust

ffmpeg:
	bash scripts/build_ffmpeg_on_mac_for_android.sh "/Users/gustavo.passos/StudioProjects/ghostr/3rd-party/ffmpeg-libs" arm64-v8a
	bash scripts/build_ffmpeg_on_mac_for_android.sh  "/Users/gustavo.passos/StudioProjects/ghostr/3rd-party/ffmpeg-libs" x86
	bash scripts/build_ffmpeg_on_mac_for_android.sh  "/Users/gustavo.passos/StudioProjects/ghostr/3rd-party/ffmpeg-libs" x86_64
	bash scripts/build_ffmpeg_on_mac_for_android.sh "/Users/gustavo.passos/StudioProjects/ghostr/3rd-party/ffmpeg-libs" armeabi-v7a


rust:
	export FFMPEG_LIBS_PATH=$(pwd)/3rd-party/ffmpeg-libs
	cd rust &&  cargo clean && cargo update &&  TARGET=android-arm64-v8a FFMPEG_LIBS_PATH=/Users/gustavo.passos/StudioProjects/ghostr/3rd-party/ffmpeg-libs/  cargo ndk -t arm64-v8a  build && cd .. && \
	TARGET=android-arm64-v8a FFMPEG_LIBS_PATH=/Users/gustavo.passos/StudioProjects/ghostr/3rd-party/ffmpeg-libs/ flutter_rust_bridge_codegen generate


rust-no-clean:
	export FFMPEG_LIBS_PATH=$(pwd)/3rd-party/ffmpeg-libs
	cd rust && cargo update &&  TARGET=android-arm64-v8a FFMPEG_LIBS_PATH=/Users/gustavo.passos/StudioProjects/ghostr/3rd-party/ffmpeg-libs/  cargo ndk -t arm64-v8a  build && cd .. && \
	TARGET=android-arm64-v8a FFMPEG_LIBS_PATH=/Users/gustavo.passos/StudioProjects/ghostr/3rd-party/ffmpeg-libs/ flutter_rust_bridge_codegen generate

gen:
	export FFMPEG_LIBS_PATH=$(pwd)/3rd-party/ffmpeg-libs
	oTARGET=android-arm64-v8a FFMPEG_LIBS_PATH=/Users/gustavo.passos/StudioProjects/ghostr/3rd-party/ffmpeg-libs/ flutter_rust_bridge_codegen generate



run: rust
	TARGET=android-arm64-v8a FFMPEG_LIBS_PATH=/Users/gustavo.passos/StudioProjects/ghostr/3rd-party/ffmpeg-libs/  flutter run $(FLAGS)

run-fast: rust-no-clean
	TARGET=android-arm64-v8a FFMPEG_LIBS_PATH=/Users/gustavo.passos/StudioProjects/ghostr/3rd-party/ffmpeg-libs/  flutter run $(FLAGS) &
	adb forward tcp:3000 tcp:3000 &


run-fast-profile: rust-no-clean
	TARGET=android-arm64-v8a FFMPEG_LIBS_PATH=/Users/gustavo.passos/StudioProjects/ghostr/3rd-party/ffmpeg-libs/  flutter run --profile $(FLAGS) &
	adb forward tcp:3000 tcp:3000 &


build: rust
	cd rust && cargo update &&  cd .. && \
	FFMPEG_LIBS_PATH=/Users/gustavo.passos/StudioProjects/ghostr/3rd-party/ffmpeg-libs/ flutter build apk --release --target-platform android-arm64
	$(MAKE) install

build-fast: rust
	FFMPEG_LIBS_PATH=/Users/gustavo.passos/StudioProjects/ghostr/3rd-party/ffmpeg-libs/ flutter build apk --release --target-platform android-arm64
	$(MAKE) install

# Define your variables at the top (these are Make variables, not shell variables)
APK_PATH := build/app/outputs/flutter-apk
VERSION  := app-release-$(shell date +'%Y-%m-%d-%H-%M').apk

install:
	mv $(APK_PATH)/app-release.apk $(APK_PATH)/$(VERSION)
	@echo "$(VERSION)"
	adb push $(APK_PATH)/$(VERSION) /sdcard/
	adb install -r $(APK_PATH)/$(VERSION)