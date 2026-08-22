import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/inline_blurhash.dart';

void main() {
  test('decodes a bounded deterministic ARGB preview raster', () {
    final black = InlineBlurHash.parse('000000');
    final example = InlineBlurHash.parse('LEHV6nWB2yk8pyo0adR*.7kCMdnj');

    expect(black.decodeArgb(width: 2, height: 2), [
      0xff000000,
      0xff000000,
      0xff000000,
      0xff000000,
    ]);
    final pixels = example.decodeArgb(width: 16, height: 16);
    expect(pixels, hasLength(256));
    expect(pixels.toSet().length, greaterThan(16));
    expect(() => example.decodeArgb(width: 33, height: 1), throwsRangeError);
    expect(() => example.decodeArgb(width: 0, height: 1), throwsRangeError);
  });
}
