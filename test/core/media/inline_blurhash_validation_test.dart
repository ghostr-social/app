import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/inline_blurhash.dart';

void main() {
  test('accepts only structurally valid bounded inline BlurHashes', () {
    final black = InlineBlurHash.parse('000000');
    final example = InlineBlurHash.tryParse('LEHV6nWB2yk8pyo0adR*.7kCMdnj');

    expect(black.horizontalComponents, 1);
    expect(black.verticalComponents, 1);
    expect(black.toString(), isNot(contains(black.encoded)));
    expect(example, isNotNull);
    expect(InlineBlurHash.tryParse(null), isNull);
    expect(InlineBlurHash.tryParse('00000!'), isNull);
    expect(InlineBlurHash.tryParse('~00000'), isNull);
    expect(InlineBlurHash.tryParse('100000'), isNull);
    expect(
      () => InlineBlurHash.parse('not-a-blurhash'),
      throwsA(isA<FormatException>()),
    );
  });
}
