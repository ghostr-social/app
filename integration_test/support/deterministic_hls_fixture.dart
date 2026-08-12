import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

/// Eight-second, 64x64 H.264 baseline fMP4 generated deterministically for
/// device playback tests. Each media fragment is exactly one second.
final class DeterministicHlsFixture {
  static final Map<String, Uint8List> assets = _decodeAssets();

  static const playlist = '''#EXTM3U
#EXT-X-VERSION:7
#EXT-X-TARGETDURATION:1
#EXT-X-MEDIA-SEQUENCE:0
#EXT-X-MAP:URI="init.mp4"
#EXTINF:1.000000,
index0.m4s
#EXTINF:1.000000,
index1.m4s
#EXTINF:1.000000,
index2.m4s
#EXTINF:1.000000,
index3.m4s
#EXTINF:1.000000,
index4.m4s
#EXTINF:1.000000,
index5.m4s
#EXTINF:1.000000,
index6.m4s
#EXTINF:1.000000,
index7.m4s
#EXT-X-ENDLIST
''';
}

Map<String, Uint8List> _decodeAssets() {
  final bundle = Uint8List.fromList(gzip.decode(base64Decode(_fixtureBundle)));
  final assets = <String, Uint8List>{};
  var offset = 0;
  for (var index = 0; index < _fixtureNames.length; index += 1) {
    final end = offset + _fixtureLengths[index];
    assets[_fixtureNames[index]] = Uint8List.sublistView(bundle, offset, end);
    offset = end;
  }
  return Map<String, Uint8List>.unmodifiable(assets);
}

const _fixtureNames = [
  'init.mp4',
  'index0.m4s',
  'index1.m4s',
  'index2.m4s',
  'index3.m4s',
  'index4.m4s',
  'index5.m4s',
  'index6.m4s',
  'index7.m4s',
];

const _fixtureLengths = [809, 866, 241, 240, 241, 240, 241, 240, 241];

const _fixtureBundle =
    'H4sIAAAAAAACE81UT2skRRSvnkniIiqjRMkhxDKTg4fNpLvnDzHYMNmwbA4KXgwrqENNd3W6na7uTlXNZGZhISsI+QBBQUEQdMWrl1wUcvDgN1gX1N1FEL0o4sFjfFWdyfaOuwmTRUh1d733qt579d771WuE0KwvB2kokjpCBaQofA2W1iyEik+zJOkhhCLWCzz0wCj+pomh3/vDeFBrVG6iE0cB3pLkpAP827Kjzyye4H28c5vHASxQTwqgszQS8j8WmjN2mRcSYDDzRnPPnLz5nSaLgRfx4U4v9GhecwPkZJ3EXkSVjlFlYewDM91j2mk+zAUv25v1OPVzoTzV5RE+4v8Wsh0B/VJI4eV0PiI913pkkZrwrANdH2o8/xroN+yKZVUs08RR2O7bjVrOYubwEGYLtNaMSwdzh/fQzCbQm8+WPjFKChM9Nf9q35sx0FTw/SWj9jUYlFIi0qMQ1FcSUtd5OEAWbk6eBvnaSLCg4yY5+WXWo32Fg+Saopz//CBdTyq83mJUUzVeymMDeHKSplHe6GKYXYCFr2SS6NoTbaydQ5F8KFJV1UjVRECjMOEF2hlQJkIVUE2EXt84Dqz5iOtZfEHt7WRyCo2lMC4xP8jjeBUuv74DUq0X0HK2ruAv7BnZIdPS9+RI9lCcbgw6k1kToX4h66c9mJ4EepNBYkDjqcnDw/d+vPz7t7/eXt//8OIP+Hb5jz8V+HgRuwmn2GrUMa/ato3b1XrDrBPYWK+AwtLrb1y+sljDqxtroOlRFzbWknQQUV9i2zSri7Zp12ExkDJdWVra3t6uqHZIIhJXEr65pE6pBJJFoJOkMkxisYJd0iauY2K48o7VwB5tR4nbcawVc8XEJCbRQFDH7IPct6oWZtTpsgCLbhs4y8SpGDiWmlvcc6yKCVYwYRb2qdfSPsGkxUm8SR27ht2AJ4y0lK26TlEUCsfGy/1lz5UQhLvFYPYo8a4lMRhYFy0L+0TIVio6YQpGRw620lbi+4JKZ9HGMuBgoRxFSdIhAQit4ZqFRRS69P6CiWOuz3BDRqSKI4wl5REBJVhvR11OBi03YSnREUGR4EqEMbgARU6Ujs8Jo8rVNg03A5kC16ED2IYQMqYFvxkQhEtj6naVI22sKsKpCFS93dZxtKDJXcflPmZtqIoKCgSnVq+YeEuF4piVBrCp8qop6TuNV4ARkqZODYcplBgABQRqgNoWwKew0Jd9ju6+X/i0PI0mnvtlB93YS1H1+j+33ulm713QmFr9eHby1fL4HdYc/eF/cUqHFU7uMFR+eIc1T+gwVM46DNWzDkMv0t0baH/3syZa3VhGF+auoOL+9p1hvrd2HifhndGEPz8l4eIpCc8/POGdkxKeP0q4dpSwBvjn+Q+QMXn1LsKDWfTMT3euD/N993HyPRgX4ImzAXxwTgA20LgAT54JYAOdD4CNsTt46kwAG+elg42xO/iJswF8TjrYGLuDL5wN4P+jgye+Kf8LetctOh8NAAA=';
