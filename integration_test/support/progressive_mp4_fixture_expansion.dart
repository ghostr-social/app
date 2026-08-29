part of 'progressive_mp4_fixture.dart';

final class _Mp4Layout {
  const _Mp4Layout({
    required this.sampleCount,
    required this.sampleEntries,
    required this.mdatStart,
    required this.mdatSize,
    required this.payloadStart,
    required this.payloadEnd,
  });

  final int sampleCount;
  final int sampleEntries;
  final int mdatStart;
  final int mdatSize;
  final int payloadStart;
  final int payloadEnd;
}

Uint8List _expandAvcSamples(Uint8List input, int extraBytes) {
  final layout = _readLayout(input);
  final extraTotal = layout.sampleCount * extraBytes;
  final output = Uint8List(input.length + extraTotal);
  output.setRange(0, layout.payloadStart, input);
  final target = _copySamples(input, output, layout, extraBytes);
  output.setRange(target, output.length, input, layout.payloadEnd);
  _writeU32(output, layout.mdatStart, layout.mdatSize + extraTotal);
  return output;
}

_Mp4Layout _readLayout(Uint8List bytes) {
  final stszType = _findBox(bytes, 'stsz');
  final mdatType = _findBox(bytes, 'mdat');
  final mdatStart = mdatType - 4;
  final mdatSize = _readU32(bytes, mdatStart);
  return _Mp4Layout(
    sampleCount: _readU32(bytes, stszType + 12),
    sampleEntries: stszType + 16,
    mdatStart: mdatStart,
    mdatSize: mdatSize,
    payloadStart: mdatType + 4,
    payloadEnd: mdatStart + mdatSize,
  );
}

int _copySamples(
  Uint8List input,
  Uint8List output,
  _Mp4Layout layout,
  int extra,
) {
  var source = layout.payloadStart;
  var target = layout.payloadStart;
  for (var index = 0; index < layout.sampleCount; index += 1) {
    final entry = layout.sampleEntries + index * 4;
    final size = _readU32(input, entry);
    output.setRange(target, target + size, input, source);
    _appendFiller(output, target + size, extra);
    _writeU32(output, entry, size + extra);
    source += size;
    target += size + extra;
  }
  return target;
}

void _appendFiller(Uint8List output, int offset, int byteLength) {
  final nalSize = byteLength - 4;
  _writeU32(output, offset, nalSize);
  output[offset + 4] = 0x0c;
  output.fillRange(offset + 5, offset + 3 + nalSize, 0xff);
  output[offset + 3 + nalSize] = 0x80;
}

int _findBox(Uint8List bytes, String box) {
  final offset = _findBoxOrNull(bytes, box);
  if (offset != null) return offset;
  throw StateError('Missing MP4 box $box.');
}

int? _findBoxOrNull(Uint8List bytes, String box) {
  final needle = ascii.encode(box);
  for (var offset = 0; offset <= bytes.length - needle.length; offset += 1) {
    var matches = true;
    for (var index = 0; index < needle.length; index += 1) {
      if (bytes[offset + index] != needle[index]) matches = false;
    }
    if (matches) return offset;
  }
  return null;
}

int _readU32(Uint8List bytes, int offset) {
  return ByteData.sublistView(bytes, offset, offset + 4).getUint32(0);
}

void _writeU32(Uint8List bytes, int offset, int value) {
  ByteData.sublistView(bytes, offset, offset + 4).setUint32(0, value);
}
