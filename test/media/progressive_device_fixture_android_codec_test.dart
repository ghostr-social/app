import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_mp4_fixture.dart';

void main() {
  test('physical Android fixture uses compatible baseline AVC', () {
    final bytes = ProgressiveMp4Fixture.sourceBytes;
    final sample = _boxStart(
      bytes,
      'avc1',
      after: _boxStart(bytes, 'stsd') + 8,
    );
    final configuration = _boxStart(bytes, 'avcC');

    expect(_uint16(bytes, sample + 32), 320);
    expect(_uint16(bytes, sample + 34), 180);
    expect(bytes.sublist(configuration + 9, configuration + 12), [
      0x42,
      0xc0,
      0x1e,
    ]);
    expect(bytes.sublist(configuration + 16, configuration + 20), [
      0x67,
      0x42,
      0xc0,
      0x1e,
    ]);
  });
}

int _boxStart(Uint8List bytes, String type, {int after = 4}) {
  final needle = ascii.encode(type);
  for (
    var offset = after;
    offset <= bytes.length - needle.length;
    offset += 1
  ) {
    if (_matches(bytes, needle, offset)) return offset - 4;
  }
  throw StateError('Missing MP4 box $type.');
}

bool _matches(Uint8List bytes, List<int> needle, int offset) {
  for (var index = 0; index < needle.length; index += 1) {
    if (bytes[offset + index] != needle[index]) return false;
  }
  return true;
}

int _uint16(Uint8List bytes, int offset) {
  return ByteData.sublistView(bytes, offset, offset + 2).getUint16(0);
}
