import 'dart:convert';

import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_mp4_fixture.dart';

void main() {
  test('device journey uses the exact expanded progressive MP4 fixture', () {
    final bytes = ProgressiveMp4Fixture.bytes;

    expect(bytes, hasLength(285652));
    expect(_contains(bytes, 'ftyp'), isTrue);
    expect(_contains(bytes, 'moov'), isTrue);
    expect(_contains(bytes, 'mdat'), isTrue);
    expect(
      sha256.convert(bytes).toString(),
      '74ddab015133a0fdb579a04fb71eb2a9b142629fce6eb55e9e87f8cf91d9592b',
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
