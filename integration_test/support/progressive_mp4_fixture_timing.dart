part of 'progressive_mp4_fixture.dart';

const _timingScalars = <({String box, int offset, bool signed})>[
  (box: 'mvhd', offset: 24, signed: false),
  (box: 'tkhd', offset: 28, signed: false),
  (box: 'elst', offset: 16, signed: false),
  (box: 'elst', offset: 20, signed: true),
  (box: 'mdhd', offset: 24, signed: false),
];

Uint8List _scaleAvcTiming(Uint8List bytes, int multiplier) {
  for (final scalar in _timingScalars) {
    _scaleScalar(bytes, scalar, multiplier);
  }
  for (final box in ['stts', 'ctts']) {
    _scaleTable(bytes, box, multiplier);
  }
  return bytes;
}

void _scaleScalar(
  Uint8List bytes,
  ({String box, int offset, bool signed}) scalar,
  int factor,
) {
  final offset = _findBox(bytes, scalar.box) - 4 + scalar.offset;
  final data = ByteData.sublistView(bytes, offset, offset + 4);
  final value = scalar.signed ? data.getInt32(0) : data.getUint32(0);
  if (scalar.signed) {
    data.setInt32(0, value * factor);
  } else {
    data.setUint32(0, value * factor);
  }
}

void _scaleTable(Uint8List bytes, String box, int factor) {
  final typeOffset = _findBoxOrNull(bytes, box);
  if (typeOffset == null) return;
  final start = typeOffset - 4;
  final count = _readU32(bytes, start + 12);
  for (var index = 0; index < count; index += 1) {
    final offset = start + 20 + index * 8;
    _writeU32(bytes, offset, _readU32(bytes, offset) * factor);
  }
}
