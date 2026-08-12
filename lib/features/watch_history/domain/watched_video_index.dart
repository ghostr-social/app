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
      byId.putIfAbsent(entry.videoId, () => entry.watchedAt);
      if (entry.mediaUrl case final String url) {
        byUrl.putIfAbsent(url, () => entry.watchedAt);
        if (inferVideoSha256FromUrl(url) case final digest?) {
          byDigest.putIfAbsent(digest.value, () => entry.watchedAt);
        }
      }
      if (entry.mediaSha256 case final String digest) {
        byDigest.putIfAbsent(digest, () => entry.watchedAt);
      }
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
      if (_byUrl[url] case final DateTime watched) return watched;
    }
    return null;
  }

  DateTime? _byAnyDigest(VideoPost post) {
    if (post.media.expectedSha256 case final declared?) {
      if (_byDigest[declared.value] case final DateTime watched) {
        return watched;
      }
    }
    for (final url in post.media.remoteUrls) {
      if (inferVideoSha256FromUrl(url) case final inferred?) {
        if (_byDigest[inferred.value] case final DateTime watched) {
          return watched;
        }
      }
    }
    return null;
  }
}
