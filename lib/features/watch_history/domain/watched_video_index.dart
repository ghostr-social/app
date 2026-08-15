import 'package:ghostr/core/media/video_identity_url.dart';
import 'package:ghostr/core/media/video_url_sha256.dart';
import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

/// Answers "has the viewer already seen this video?" across every identity
/// a clip can travel under: its event coordinate, its media URLs, and its
/// file digest.
class WatchedVideoIndex {
  factory WatchedVideoIndex(List<WatchHistoryEntry> entries) {
    final byId = <String, DateTime>{};
    final byUrl = <String, DateTime>{};
    final byDigest = <String, DateTime>{};
    for (final entry in entries) {
      _indexEntry(entry, byId, byUrl, byDigest);
    }
    return WatchedVideoIndex._(byId, byUrl, byDigest);
  }

  const WatchedVideoIndex._(this._byId, this._byUrl, this._byDigest);

  final Map<String, DateTime> _byId;
  final Map<String, DateTime> _byUrl;
  final Map<String, DateTime> _byDigest;

  bool get isEmpty => _byId.isEmpty;

  bool contains(VideoPost post) => watchedAt(post) != null;

  DateTime? watchedAt(VideoPost post) {
    return _byId[VideoInteractionTarget.fromPost(post).value] ??
        _byAnyUrl(post) ??
        _byAnyDigest(post);
  }

  DateTime? _byAnyUrl(VideoPost post) {
    for (final url in post.media.remoteUrls) {
      final identityUrl = canonicalVideoIdentityUrl(url);
      if (_byUrl[identityUrl] case final DateTime watched) return watched;
    }
    return null;
  }

  DateTime? _byAnyDigest(VideoPost post) {
    final declared = _byDeclaredDigest(post);
    if (declared != null) return declared;
    for (final url in post.media.remoteUrls) {
      final watched = _byInferredDigest(url);
      if (watched != null) return watched;
    }
    return null;
  }

  DateTime? _byDeclaredDigest(VideoPost post) {
    final declared = post.media.expectedSha256;
    return declared == null ? null : _byDigest[declared.value];
  }

  DateTime? _byInferredDigest(String url) {
    final inferred = inferVideoSha256FromUrl(url);
    return inferred == null ? null : _byDigest[inferred.value];
  }

  static void _indexEntry(
    WatchHistoryEntry entry,
    Map<String, DateTime> byId,
    Map<String, DateTime> byUrl,
    Map<String, DateTime> byDigest,
  ) {
    byId.putIfAbsent(entry.videoId, () => entry.watchedAt);
    for (final url in entry.mediaUrls) {
      _indexUrl(url, entry.watchedAt, byUrl, byDigest);
    }
    final declared = entry.mediaSha256;
    if (declared != null) {
      byDigest.putIfAbsent(declared, () => entry.watchedAt);
    }
  }

  static void _indexUrl(
    String url,
    DateTime watchedAt,
    Map<String, DateTime> byUrl,
    Map<String, DateTime> byDigest,
  ) {
    final identityUrl = canonicalVideoIdentityUrl(url);
    if (identityUrl.isNotEmpty) {
      byUrl.putIfAbsent(identityUrl, () => watchedAt);
    }
    if (inferVideoSha256FromUrl(url) case final digest?) {
      byDigest.putIfAbsent(digest.value, () => watchedAt);
    }
  }
}
