const _alphabet = 'qpzry9x8gf2tvdw0s3jn54khce6mua7l';
const _generators = <int>[
  0x3b6a57b2,
  0x26508e6d,
  0x1ea119fa,
  0x3d4233dd,
  0x2a1462b3,
];

List<int>? decodeNostrBech32Key(String raw, String expectedHrp) {
  final value = raw.trim();
  if (!_hasValidEnvelope(value, expectedHrp)) return null;
  final values = _decodeCharacters(value.substring(expectedHrp.length + 1));
  if (values == null) return null;
  if (!_hasValidChecksum(expectedHrp, values)) return null;
  return _decodePayload(values.sublist(0, values.length - 6));
}

bool _hasValidEnvelope(String value, String expectedHrp) {
  return value == value.toLowerCase() &&
      value.startsWith('${expectedHrp}1') &&
      value.length == expectedHrp.length + 59;
}

String nostrKeyHex(List<int> bytes) {
  return bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join();
}

List<int>? nostrKeyBytes(String raw) {
  final value = raw.trim().toLowerCase();
  if (value.length != 64) return null;
  final bytes = <int>[];
  for (var index = 0; index < value.length; index += 2) {
    final byte = int.tryParse(value.substring(index, index + 2), radix: 16);
    if (byte == null) return null;
    bytes.add(byte);
  }
  return bytes;
}

String? encodeNostrBech32Key(String hrp, List<int> bytes) {
  if (bytes.length != 32) return null;
  if (bytes.any((byte) => byte < 0 || byte > 255)) return null;
  final words = _encodePayload(bytes);
  final payload = [...words, ..._checksum(hrp, words)];
  return '${hrp}1${payload.map((value) => _alphabet[value]).join()}';
}

List<int> _encodePayload(List<int> bytes) {
  var accumulator = 0;
  var bitCount = 0;
  final words = <int>[];
  for (final byte in bytes) {
    accumulator = (accumulator << 8) | byte;
    bitCount += 8;
    while (bitCount >= 5) {
      bitCount -= 5;
      words.add((accumulator >> bitCount) & 31);
    }
  }
  if (bitCount > 0) words.add((accumulator << (5 - bitCount)) & 31);
  return words;
}

List<int> _checksum(String hrp, List<int> words) {
  final polymod = _polymod([
        ..._expandedHrp(hrp),
        ...words,
        ...List<int>.filled(6, 0),
      ]) ^
      1;
  return [
    for (var index = 0; index < 6; index += 1)
      (polymod >> (5 * (5 - index))) & 31,
  ];
}

List<int>? _decodeCharacters(String encoded) {
  final values = encoded.split('').map(_alphabet.indexOf).toList();
  return values.any((value) => value < 0) ? null : values;
}

bool _hasValidChecksum(String hrp, List<int> values) {
  return _polymod([..._expandedHrp(hrp), ...values]) == 1;
}

List<int>? _decodePayload(List<int> values) {
  if (!_hasValidPayloadShape(values)) return null;
  final decoded = _convertPayload(values);
  return _hasValidDecodedPayload(decoded) ? decoded.bytes : null;
}

bool _hasValidPayloadShape(List<int> values) {
  return values.length == 52 && (values.last & 15) == 0;
}

({List<int> bytes, int remainder}) _convertPayload(List<int> values) {
  var accumulator = 0;
  var bitCount = 0;
  final bytes = <int>[];
  for (final value in values) {
    accumulator = (accumulator << 5) | value;
    bitCount += 5;
    if (bitCount < 8) continue;
    bitCount -= 8;
    bytes.add((accumulator >> bitCount) & 255);
    accumulator &= (1 << bitCount) - 1;
  }
  return (bytes: bytes, remainder: accumulator);
}

bool _hasValidDecodedPayload(({List<int> bytes, int remainder}) decoded) {
  return decoded.bytes.length == 32 && decoded.remainder == 0;
}

Iterable<int> _expandedHrp(String value) sync* {
  for (final code in value.codeUnits) {
    yield code >> 5;
  }
  yield 0;
  for (final code in value.codeUnits) {
    yield code & 31;
  }
}

int _polymod(Iterable<int> values) {
  var checksum = 1;
  for (final value in values) {
    final top = checksum >> 25;
    checksum = (checksum & 0x1ffffff) << 5 ^ value;
    for (var index = 0; index < _generators.length; index += 1) {
      if ((top >> index) & 1 == 1) checksum ^= _generators[index];
    }
  }
  return checksum;
}
