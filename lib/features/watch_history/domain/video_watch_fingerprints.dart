import 'dart:convert';

import 'package:crypto/crypto.dart';
import 'package:ghostr/core/media/video_identity_url.dart';
import 'package:ghostr/core/media/video_sha256.dart';
import 'package:ghostr/core/media/video_url_sha256.dart';
import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

final class VideoWatchFingerprints {
  factory VideoWatchFingerprints.fromPost(VideoPost post) {
    return VideoWatchFingerprints._from(
      VideoInteractionTarget.fromPost(post).value,
      post.media.remoteUrls,
      post.media.expectedSha256?.value,
    );
  }

  factory VideoWatchFingerprints.fromEntry(WatchHistoryEntry entry) {
    return VideoWatchFingerprints._from(
      entry.videoId,
      entry.mediaUrls,
      entry.mediaSha256,
    );
  }

  factory VideoWatchFingerprints.stored({
    required String videoId,
    required List<String> mediaUrls,
    String? mediaSha256,
  }) {
    return VideoWatchFingerprints._from(videoId, mediaUrls, mediaSha256);
  }

  factory VideoWatchFingerprints._from(
    String videoId,
    List<String> mediaUrls,
    String? mediaSha256,
  ) {
    final target = _requiredTarget(videoId);
    final values = <String>{_fingerprint('target', target)};
    for (final rawUrl in mediaUrls) {
      _addUrlFingerprints(values, rawUrl);
    }
    _addDeclaredDigest(values, mediaSha256);
    return VideoWatchFingerprints._(List<String>.unmodifiable(values));
  }

  const VideoWatchFingerprints._(this.values);

  final List<String> values;

  String get target => values.first;

  static String _requiredTarget(String videoId) {
    final target = videoId.trim();
    if (target.isEmpty) {
      throw const FormatException('Watched-video identity is empty.');
    }
    return target;
  }

  static void _addUrlFingerprints(Set<String> values, String rawUrl) {
    final url = canonicalVideoIdentityUrl(rawUrl);
    if (url.isEmpty) return;
    values.add(_fingerprint('url', url));
    if (inferVideoSha256FromUrl(url) case final digest?) {
      values.add(_fingerprint('digest', digest.value));
    }
  }

  static void _addDeclaredDigest(Set<String> values, String? rawDigest) {
    if (rawDigest == null) return;
    final digest = VideoSha256.tryParse(rawDigest);
    if (digest == null) return;
    values.add(_fingerprint('digest', digest.value));
  }

  static String _fingerprint(String kind, String value) {
    final digest = sha256.convert(utf8.encode('$kind\u0000$value')).toString();
    return digest.substring(0, 32);
  }
}
