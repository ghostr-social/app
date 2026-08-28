import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/deterministic_hls_fixture.dart';

void main() {
  test('HLS fixture pins 320x180 constrained-baseline fragments', () {
    final assets = DeterministicHlsFixture.assets;
    final init = assets['init.mp4'];

    expect(init, isNotNull);
    expect(_trackDimensions(init!), (width: 320, height: 180));
    expect(_avcProfile(init), [0x42, 0xc0, 0x1e]);
    expect(_boxTypes(init), containsAllInOrder(['ftyp', 'moov']));
    expect(assets.keys, orderedEquals(_assetHashes.keys));
    for (final entry in assets.entries) {
      expect(sha256.convert(entry.value).toString(), _assetHashes[entry.key]);
      if (entry.key == 'init.mp4') continue;
      expect(_boxTypes(entry.value), ['styp', 'sidx', 'moof', 'mdat']);
      expect(DeterministicHlsFixture.playlist, contains(entry.key));
    }
    expect('#EXTINF:1.000000,'.allMatchesInPlaylist(), hasLength(8));
  });
}

List<int> _avcProfile(Uint8List bytes) {
  final offset = latin1.decode(bytes).indexOf('avcC');
  if (offset < 0) throw StateError('Missing avcC box.');
  return bytes.sublist(offset + 5, offset + 8);
}

List<String> _boxTypes(Uint8List bytes) {
  final types = <String>[];
  var offset = 0;
  while (offset + 8 <= bytes.length) {
    final size = _readU32(bytes, offset);
    if (size < 8 || offset + size > bytes.length) break;
    types.add(latin1.decode(bytes.sublist(offset + 4, offset + 8)));
    offset += size;
  }
  return types;
}

({int width, int height}) _trackDimensions(Uint8List bytes) {
  final typeOffset = latin1.decode(bytes).indexOf('tkhd');
  if (typeOffset < 4) throw StateError('Missing tkhd box.');
  final boxStart = typeOffset - 4;
  final boxSize = _readU32(bytes, boxStart);
  return (
    width: _readU32(bytes, boxStart + boxSize - 8) ~/ 65536,
    height: _readU32(bytes, boxStart + boxSize - 4) ~/ 65536,
  );
}

int _readU32(Uint8List bytes, int offset) {
  return ByteData.sublistView(bytes, offset, offset + 4).getUint32(0);
}

extension on String {
  Iterable<RegExpMatch> allMatchesInPlaylist() {
    return RegExp(
      RegExp.escape(this),
    ).allMatches(DeterministicHlsFixture.playlist);
  }
}

const _assetHashes = {
  'init.mp4':
      '32a8871c0ce2a9e2aa63dfeb09fb59dc2847f595c38ea77c731298d62175fe22',
  'index0.m4s':
      'bf9639f14b6c797c935d21621fb1636118a0099127649ab7c6405e07913f9722',
  'index1.m4s':
      'f9c6245e27161eb69b8befad4d425d517f94e5d3e232b00b3525054439d90c3c',
  'index2.m4s':
      '118abf64bf00260b2924d4f77b09ba6ddc506e5b725d41b5907c57e8829bd112',
  'index3.m4s':
      'bf34b1292ea4f4fab894bcc944c3c93b8e12122f19902447d2a77bd78bf443ab',
  'index4.m4s':
      'ec05a78281a3890f0430647f95d2b82b017c5d0294efe09f33499f339ff2815e',
  'index5.m4s':
      'ba498ed2e1a398e9f0a595cfabdad45a6b0225a38b50877bf478e49aebc9fdd1',
  'index6.m4s':
      '8adbef01b461039476e756eb8fd6fe355ba52557cb940253177c2296606d30d4',
  'index7.m4s':
      'c30b2369a5d5e86af2b67ddb76939f91c505deb6ce9dee7f437f0b9916e0bbe1',
};
