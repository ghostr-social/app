import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

/// Answers "has the viewer already seen this video?" across every identity
/// a clip can travel under: its event coordinate, its media URL, and its
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
        _lookup(_byUrl, post.media.remoteUrl) ??
        _lookup(_byDigest, post.media.expectedSha256?.value);
  }

  DateTime? _lookup(Map<String, DateTime> times, String? key) {
    return key == null ? null : times[key];
  }
}
