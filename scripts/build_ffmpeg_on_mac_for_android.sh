#!/usr/bin/env bash
set -eo pipefail

################################################################################
# Configuration
################################################################################
CFLAGS=""
LDFLAGS=""

# Default API level if not defined externally
: "${API:=21}"

# Default NDK path if not defined externally (Adjust NDK version/path as needed)
NDK_PATH="${NDK:-"$HOME/Library/Android/sdk/ndk/26.1.10909125"}"
HOST_TAG="darwin-x86_64"

EXTRA_CONFIG=()
# The first argument is the output directory.
# The second argument is the Android architecture (e.g. arm64-v8a, armeabi-v7a, x86, x86_64).
OUTPUT_DIR="${1:-"$(pwd)/ffmpeg-libs"}"
ANDROID_ABI="${2:-"arm64-v8a"}"   # If not provided, default to arm64-v8a

# We’ll map ANDROID_ABI -> the correct config for FFmpeg
case "$ANDROID_ABI" in
  arm64-v8a)
    FFMPEG_ARCH="arm64"
    FFMPEG_CPU="armv8-a"
    ANDROID_TRIPLE="aarch64-linux-android"
    ;;
  armeabi-v7a)
    FFMPEG_ARCH="arm"
    FFMPEG_CPU="armv7-a"
    ANDROID_TRIPLE="armv7a-linux-androideabi"
    ;;
  x86)
    FFMPEG_ARCH="x86"
    FFMPEG_CPU="i686"
    ANDROID_TRIPLE="i686-linux-android"
    CFLAGS="-fPIC"
    LDFLAGS="-fPIC"
    EXTRA_CONFIG+=("--disable-inline-asm")
    EXTRA_CONFIG+=("--disable-asm")
    export ASFLAGS="-f PIC"
    export NASMFLAGS="-f PIC"

    ;;
  x86_64)
    FFMPEG_ARCH="x86_64"
    FFMPEG_CPU="x86-64"
    ANDROID_TRIPLE="x86_64-linux-android"
    ;;
  *)
    echo "ERROR: Unknown or unsupported Android ABI: $ANDROID_ABI"
    echo "Supported: arm64-v8a, armeabi-v7a, x86, x86_64"
    exit 1
    ;;
esac

################################################################################
# Preliminary checks
################################################################################

# Make sure the NDK path exists
if [[ ! -d "$NDK_PATH" ]]; then
  echo "ERROR: Android NDK not found at: $NDK_PATH"
  exit 1
fi

# Required commands
for cmd in git sysctl; do
  if ! command -v "$cmd" &>/dev/null; then
    echo "ERROR: '$cmd' is required but not found in PATH."
    exit 1
  fi
done

################################################################################
# Create a temporary workspace
################################################################################

BUILD_DIR="$(mktemp -d -t ffmpeg-build-XXXX)"
FFMPEG_SRC="$BUILD_DIR/ffmpeg"

# We’ll install into "$BUILD_DIR/$ANDROID_ABI" then copy to OUTPUT_DIR at the end
INSTALL_PREFIX="$BUILD_DIR/$ANDROID_ABI"

echo "==> Created temporary build directory: $BUILD_DIR"
mkdir -p "$INSTALL_PREFIX"

################################################################################
# Toolchain configuration
################################################################################

TOOLCHAIN="$NDK_PATH/toolchains/llvm/prebuilt/$HOST_TAG"
SYSROOT="$TOOLCHAIN/sysroot"

CC="$TOOLCHAIN/bin/${ANDROID_TRIPLE}${API}-clang"
CXX="$TOOLCHAIN/bin/${ANDROID_TRIPLE}${API}-clang++"
AR="$TOOLCHAIN/bin/llvm-ar"
LD="$TOOLCHAIN/bin/ld.lld"
STRIP="$TOOLCHAIN/bin/llvm-strip"
NM="$TOOLCHAIN/bin/llvm-nm"
RANLIB="$TOOLCHAIN/bin/llvm-ranlib"
export PATH="$TOOLCHAIN/bin:$PATH"

if ! command -v "$CC" &>/dev/null; then
  echo "ERROR: Compiler not found at $CC"
  exit 1
fi

echo "==> Using compiler: $(which "$CC")"
"$CC" --version

################################################################################
# Fetch FFmpeg source
################################################################################

echo "==> Cloning FFmpeg into $FFMPEG_SRC"
git clone --depth=1 https://github.com/FFmpeg/FFmpeg.git "$FFMPEG_SRC"

################################################################################
# Configure FFmpeg
################################################################################

echo "==> Configuring FFmpeg for $ANDROID_ABI..."
cd "$FFMPEG_SRC"

#!/usr/bin/env bash

# Define your configure arguments in an array
CONFIGURE_ARGS=(
  --prefix="$INSTALL_PREFIX"
  --enable-cross-compile
  --target-os=android
  --arch="$FFMPEG_ARCH"
  --cpu="$FFMPEG_CPU"
  --cc="$CC"
  --extra-cflags="$CFLAGS"
  --extra-ldflags="$LDFLAGS"
  --cxx="$CXX"
  --nm="$NM"
  --ranlib="$RANLIB"
  --ar="$AR"
  --strip="$STRIP"
  --sysroot="$SYSROOT"
  --cross-prefix="$TOOLCHAIN/bin/${ANDROID_TRIPLE}-"
  --enable-pic
  --enable-static
  --disable-shared
  --disable-programs
  --disable-doc
  --disable-symver
  --enable-gpl
  --enable-nonfree
  --enable-pthreads
  "${EXTRA_CONFIG[@]}"
  --disable-zlib
)

echo "Running configure with arguments:"
printf '%s\n' "${CONFIGURE_ARGS[@]}"

# Now run the actual configure command with the expanded arguments:
./configure "${CONFIGURE_ARGS[@]}"


################################################################################
# Build & Install
################################################################################

echo "==> Building FFmpeg..."
make -j"$(sysctl -n hw.logicalcpu)"

echo "==> Installing FFmpeg to $INSTALL_PREFIX..."
make install

################################################################################
# Copy artifacts to the output directory
################################################################################

echo "==> Copying artifacts to $OUTPUT_DIR/$ANDROID_ABI ..."
mkdir -p "$OUTPUT_DIR/$ANDROID_ABI"

# The 'include' and 'lib' directories inside INSTALL_PREFIX contain the headers and .a files
cp -R "$INSTALL_PREFIX/include" "$OUTPUT_DIR/$ANDROID_ABI"
cp -R "$INSTALL_PREFIX/lib"     "$OUTPUT_DIR/$ANDROID_ABI"

echo
echo "FFmpeg build artifacts for $ANDROID_ABI are now located in: $OUTPUT_DIR/$ANDROID_ABI"
echo
echo "==> Cleaning up temporary directory $BUILD_DIR..."
rm -rf "$BUILD_DIR"

echo "==> Done."
