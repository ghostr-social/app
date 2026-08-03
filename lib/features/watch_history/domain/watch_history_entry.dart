import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class WatchHistoryEntry {
  WatchHistoryEntry({
    required String videoId,
    required String title,
    required String creatorName,
    required this.watchedAt,
  })  : videoId = _requireVideoId(videoId),
        title = title.trim(),
        creatorName = creatorName.trim();

  factory WatchHistoryEntry.fromPost(VideoPost post, DateTime watchedAt) {
    final caption = post.caption.trim();
    return WatchHistoryEntry(
      videoId: VideoInteractionTarget.fromPost(post).value,
      title: caption.isEmpty ? post.songName : caption,
      creatorName: post.creator.displayName,
      watchedAt: watchedAt,
    );
  }

  final String videoId;
  final String title;
  final String creatorName;
  final DateTime watchedAt;

  static String _requireVideoId(String value) {
    final trimmed = value.trim();
    if (trimmed.isEmpty) {
      throw const FormatException('Watch history video id cannot be empty.');
    }
    return trimmed;
  }
}
