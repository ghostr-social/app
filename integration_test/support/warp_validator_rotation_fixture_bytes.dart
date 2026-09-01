part of 'warp_validator_rotation_fixture.dart';

Uint8List _rotationFirstBytes() {
  return Uint8List.fromList(ProgressiveMp4Fixture.bytes);
}

Uint8List _rotationSecondBytes(Uint8List first) {
  final bytes = Uint8List.fromList(first);
  final marker = ascii.encode('Lavf62.3.100');
  final offset = _rotationIndexOf(bytes, marker);
  if (offset < 0) throw StateError('MP4 encoder marker is missing.');
  bytes[offset + marker.length - 1] = '1'.codeUnitAt(0);
  return bytes;
}

int _rotationIndexOf(Uint8List bytes, List<int> marker) {
  for (var offset = 0; offset <= bytes.length - marker.length; offset += 1) {
    var matches = true;
    for (var index = 0; index < marker.length; index += 1) {
      if (bytes[offset + index] != marker[index]) matches = false;
    }
    if (matches) return offset;
  }
  return -1;
}
