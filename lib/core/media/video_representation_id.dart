import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:ghostr/core/media/video_media_source.dart';

/// Cross-language identity of one exact remote-media representation.
final class VideoRepresentationId {
  const VideoRepresentationId._(this.value);

  factory VideoRepresentationId.parse(String raw) {
    if (!_pattern.hasMatch(raw)) {
      throw const FormatException('Invalid video representation id.');
    }
    return VideoRepresentationId._(raw);
  }

  factory VideoRepresentationId.forMedia(VideoMediaSource media) {
    final delivery = media.remoteDelivery;
    if (delivery == null || media.cacheSourceUrls.isEmpty) {
      throw ArgumentError.value(media, 'media', 'Remote media is required.');
    }
    final bytes = _RepresentationBytes(delivery);
    final advertised = media.expectedSha256?.value;
    advertised == null
        ? bytes.addUnverified(media)
        : bytes.addField(advertised);
    return VideoRepresentationId._(bytes.digest);
  }

  final String value;

  @override
  bool operator ==(Object other) {
    return other is VideoRepresentationId && other.value == value;
  }

  @override
  int get hashCode => value.hashCode;
}

final class _RepresentationBytes {
  _RepresentationBytes(VideoMediaDelivery delivery) {
    _bytes.addByte(delivery == VideoMediaDelivery.progressive ? 0 : 1);
  }

  final _bytes = BytesBuilder(copy: false);

  String get digest => sha256.convert(_bytes.takeBytes()).toString();

  void addUnverified(VideoMediaSource media) {
    final urls = media.cacheSourceUrls.toSet().toList()..sort(_compareUtf8);
    for (final url in urls) {
      addField(url);
    }
    final metadata = media.mediaMetadata;
    _addOptionalNumber(metadata.sizeBytes);
    _addOptionalNumber(metadata.durationMs);
  }

  void addField(String value) {
    final encoded = utf8.encode(value);
    _bytes.add(_u64(encoded.length));
    _bytes.add(encoded);
  }

  void _addOptionalNumber(int? value) {
    if (value != null && value < 0) {
      throw ArgumentError.value(value, 'value', 'Must not be negative.');
    }
    _bytes.addByte(value == null ? 0 : 1);
    _bytes.add(_u64(value ?? 0));
  }
}

int _compareUtf8(String left, String right) {
  final leftBytes = utf8.encode(left);
  final rightBytes = utf8.encode(right);
  final shared = leftBytes.length < rightBytes.length
      ? leftBytes.length
      : rightBytes.length;
  for (var index = 0; index < shared; index += 1) {
    final compared = leftBytes[index].compareTo(rightBytes[index]);
    if (compared != 0) return compared;
  }
  return leftBytes.length.compareTo(rightBytes.length);
}

Uint8List _u64(int value) {
  final data = ByteData(8)..setUint64(0, value, Endian.big);
  return data.buffer.asUint8List();
}

final _pattern = RegExp(r'^[0-9a-f]{64}$');
