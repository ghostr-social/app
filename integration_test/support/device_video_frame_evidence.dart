import 'dart:math' as math;
import 'dart:typed_data';
import 'dart:ui' as ui;

final class DeviceVideoFrameEvidence {
  const DeviceVideoFrameEvidence({
    required this.chromaticRatio,
    required this.changedRatio,
  });

  static Future<DeviceVideoFrameEvidence> compare(
    List<int> firstPng,
    List<int> secondPng,
  ) async {
    final first = await _FramePixels.decode(firstPng);
    final second = await _FramePixels.decode(secondPng);
    if (first.width != second.width || first.height != second.height) {
      throw StateError('Video evidence frame dimensions changed.');
    }
    return _measure(first, second);
  }

  static DeviceVideoFrameEvidence _measure(
    _FramePixels first,
    _FramePixels second,
  ) {
    final region = _FrameRegion.center(first.width, first.height);
    var firstChromatic = 0;
    var secondChromatic = 0;
    var changed = 0;
    var sampled = 0;
    for (final offset in region.offsets()) {
      if (_isChromatic(first.bytes, offset)) firstChromatic += 1;
      if (_isChromatic(second.bytes, offset)) secondChromatic += 1;
      if (_hasChanged(first.bytes, second.bytes, offset)) changed += 1;
      sampled += 1;
    }
    return DeviceVideoFrameEvidence(
      chromaticRatio: math.min(firstChromatic, secondChromatic) / sampled,
      changedRatio: changed / sampled,
    );
  }

  static bool _isChromatic(Uint8List bytes, int offset) {
    final red = bytes[offset];
    final green = bytes[offset + 1];
    final blue = bytes[offset + 2];
    final high = math.max(red, math.max(green, blue));
    final low = math.min(red, math.min(green, blue));
    return high >= 64 && high - low >= 32;
  }

  static bool _hasChanged(Uint8List first, Uint8List second, int offset) {
    var difference = 0;
    for (var channel = 0; channel < 3; channel += 1) {
      difference += (first[offset + channel] - second[offset + channel]).abs();
    }
    return difference >= 48;
  }

  final double chromaticRatio;
  final double changedRatio;
}

final class _FramePixels {
  const _FramePixels(this.bytes, this.width, this.height);

  static Future<_FramePixels> decode(List<int> png) async {
    final codec = await ui.instantiateImageCodec(Uint8List.fromList(png));
    final frame = await codec.getNextFrame();
    final image = frame.image;
    final data = await image.toByteData(format: ui.ImageByteFormat.rawRgba);
    final pixels = data!.buffer.asUint8List(
      data.offsetInBytes,
      data.lengthInBytes,
    );
    final result = _FramePixels(pixels, image.width, image.height);
    image.dispose();
    codec.dispose();
    return result;
  }

  final Uint8List bytes;
  final int width;
  final int height;
}

final class _FrameRegion {
  const _FrameRegion(this.width, this.height);

  factory _FrameRegion.center(int width, int height) {
    return _FrameRegion(width, height);
  }

  final int width;
  final int height;

  int get left => width * 3 ~/ 10;
  int get right => width * 7 ~/ 10;
  int get top => height * 2 ~/ 10;
  int get bottom => height * 8 ~/ 10;

  Iterable<int> offsets() sync* {
    for (var y = top; y < bottom; y += 2) {
      for (var x = left; x < right; x += 2) {
        yield (y * width + x) * 4;
      }
    }
  }
}
