import 'dart:convert';
import 'dart:typed_data';

import 'deterministic_hls_fixture.dart';

final class ProgressiveDeviceHlsAssets {
  const ProgressiveDeviceHlsAssets._();

  static Uint8List? resolve(String id, String asset) {
    if (id == 'multivariant') return _multivariant(asset);
    if (asset == 'index.m3u8') return _playlist();
    return DeterministicHlsFixture.assets[asset];
  }

  static Uint8List? _multivariant(String asset) {
    if (asset == 'index.m3u8') return _bytes(_multivariantMaster);
    if (asset == 'selected.m3u8' || asset == 'alternate.m3u8') {
      return _playlist();
    }
    return DeterministicHlsFixture.assets[asset];
  }

  static Uint8List _playlist() => _bytes(DeterministicHlsFixture.playlist);

  static Uint8List _bytes(String value) =>
      Uint8List.fromList(utf8.encode(value));
}

const _multivariantMaster = '''#EXTM3U
#EXT-X-STREAM-INF:BANDWIDTH=1000000
selected.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=5000000
alternate.m3u8
''';
