import 'dart:typed_data';

import 'deterministic_hls_fixture.dart';
import 'progressive_device_origin.dart';

final class WarpNoVideoRenditionFixture {
  WarpNoVideoRenditionFixture._(this._target, this._original);

  static WarpNoVideoRenditionFixture install() {
    final target = DeterministicHlsFixture.assets['init.mp4']!;
    final original = Uint8List.fromList(target);
    target.fillRange(0, target.length, 0);
    _writeWaveHeader(target);
    return WarpNoVideoRenditionFixture._(target, original);
  }

  final Uint8List _target;
  final Uint8List _original;
  var _restored = false;

  Uri urlFor(ProgressiveDeviceOrigin origin) {
    return origin.hlsUrlFor('invalid-track').resolve('init.mp4');
  }

  void restore() {
    if (_restored) return;
    _restored = true;
    _target.setAll(0, _original);
  }
}

void _writeWaveHeader(Uint8List bytes) {
  final values = ByteData.sublistView(bytes);
  _writeText(bytes, 0, 'RIFF');
  values.setUint32(4, bytes.length - 8, Endian.little);
  _writeText(bytes, 8, 'WAVEfmt ');
  values.setUint32(16, 16, Endian.little);
  values.setUint16(20, 1, Endian.little);
  values.setUint16(22, 1, Endian.little);
  values.setUint32(24, 8000, Endian.little);
  values.setUint32(28, 16000, Endian.little);
  values.setUint16(32, 2, Endian.little);
  values.setUint16(34, 16, Endian.little);
  _writeText(bytes, 36, 'data');
  values.setUint32(40, bytes.length - 44, Endian.little);
}

void _writeText(Uint8List bytes, int offset, String text) {
  bytes.setRange(offset, offset + text.length, text.codeUnits);
}
