import 'dart:math' as math;

part 'inline_blurhash_raster.dart';

const _base83 =
    '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz'
    r'#$%*+,-.:;=?@[]^_{|}~';
final _base83CodeUnits = _base83.codeUnits;

/// A structurally validated, bounded inline BlurHash value.
final class InlineBlurHash {
  factory InlineBlurHash.parse(String encoded) {
    final parsed = tryParse(encoded);
    if (parsed == null) {
      throw const FormatException('Invalid inline BlurHash.');
    }
    return parsed;
  }

  static InlineBlurHash? tryParse(String? encoded) {
    if (encoded == null) return null;
    final sizeFlag = _base83At(encoded, 0);
    if (sizeFlag == null || sizeFlag > 80) return null;
    final horizontal = sizeFlag % 9 + 1;
    final vertical = sizeFlag ~/ 9 + 1;
    if (!_hasValidShape(encoded, horizontal, vertical)) return null;
    return InlineBlurHash._(encoded, horizontal, vertical);
  }

  const InlineBlurHash._(
    this.encoded,
    this.horizontalComponents,
    this.verticalComponents,
  );

  final String encoded;
  final int horizontalComponents;
  final int verticalComponents;

  List<int> decodeArgb({required int width, required int height}) {
    _validateRasterSize(width, height);
    return _BlurHashDecoder(this, width, height).decode();
  }

  @override
  bool operator ==(Object other) {
    return other is InlineBlurHash && other.encoded == encoded;
  }

  @override
  int get hashCode => encoded.hashCode;
}

bool _hasValidShape(String encoded, int horizontal, int vertical) {
  if (encoded.length != 4 + 2 * horizontal * vertical) return false;
  return encoded.codeUnits.every((unit) => _base83CodeUnits.contains(unit));
}

int? _base83At(String encoded, int index) {
  if (index >= encoded.length) return null;
  final value = _base83CodeUnits.indexOf(encoded.codeUnitAt(index));
  return value < 0 ? null : value;
}

int _decode83(String encoded, int start, int length) {
  var value = 0;
  for (var index = start; index < start + length; index += 1) {
    value = value * 83 + _base83CodeUnits.indexOf(encoded.codeUnitAt(index));
  }
  return value;
}

void _validateRasterSize(int width, int height) {
  const maximumDimension = 32;
  const maximumPixels = 1024;
  final inRange =
      width > 0 &&
      height > 0 &&
      width <= maximumDimension &&
      height <= maximumDimension;
  if (!inRange || width * height > maximumPixels) {
    throw RangeError('Inline preview raster dimensions are out of range.');
  }
}
