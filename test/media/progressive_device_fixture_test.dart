import 'dart:convert';

import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_mp4_fixture.dart';

void main() {
  test('device journey uses the exact expanded progressive MP4 fixture', () {
    final bytes = ProgressiveMp4Fixture.bytes;

    expect(bytes, hasLength(293999));
    expect(_contains(bytes, 'ftyp'), isTrue);
    expect(_contains(bytes, 'moov'), isTrue);
    expect(_contains(bytes, 'mdat'), isTrue);
    expect(
      sha256.convert(bytes).toString(),
      'd5c6f10986d2a23730172e1b74e8ee7e9dd976aefe673570e719029ec3bd4ddc',
    );
  });
}

bool _contains(List<int> bytes, String box) {
  final needle = ascii.encode(box);
  for (var offset = 0; offset <= bytes.length - needle.length; offset += 1) {
    if (_matches(bytes, needle, offset)) return true;
  }
  return false;
}

bool _matches(List<int> bytes, List<int> needle, int offset) {
  for (var index = 0; index < needle.length; index += 1) {
    if (bytes[offset + index] != needle[index]) return false;
  }
  return true;
}
