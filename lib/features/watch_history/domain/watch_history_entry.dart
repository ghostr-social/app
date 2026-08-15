import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class WatchHistoryEntry {
  WatchHistoryEntry({
    required String videoId,
    required String title,
    required String creatorName,
    required this.watchedAt,
    List<String> mediaUrls = const <String>[],
    String? mediaUrl,
    String? mediaSha256,
  }) : videoId = _requireVideoId(videoId),
       title = title.trim(),
       creatorName = creatorName.trim(),
       mediaUrls = _mediaUrls(mediaUrls, mediaUrl),
       mediaSha256 = _optional(mediaSha256);

  factory WatchHistoryEntry.fromPost(VideoPost post, DateTime watchedAt) {
    final caption = post.caption.trim();
    return WatchHistoryEntry(
      videoId: VideoInteractionTarget.fromPost(post).value,
      title: caption.isEmpty ? post.songName : caption,
      creatorName: post.creator.displayName,
      watchedAt: watchedAt,
      mediaUrls: post.media.remoteUrls,
      mediaSha256: post.media.expectedSha256?.value,
    );
  }

  final String videoId;
  final String title;
  final String creatorName;
  final DateTime watchedAt;

  /// Where the video file was streamed from, so republishes of the same
  /// URL under a new event id still count as watched.
  final List<String> mediaUrls;

  String? get mediaUrl => mediaUrls.firstOrNull;

  /// The file digest when the event declared one, so the same file on a
  /// different host still counts as watched.
  final String? mediaSha256;

  static String _requireVideoId(String value) {
    final trimmed = value.trim();
    if (trimmed.isEmpty) {
      throw const FormatException('Watch history video id cannot be empty.');
    }
    return trimmed;
  }

  static String? _optional(String? value) {
    final trimmed = value?.trim();
    return trimmed == null || trimmed.isEmpty ? null : trimmed;
  }

  static List<String> _mediaUrls(List<String> urls, String? legacyUrl) {
    final values = <String>{
      for (final url in urls)
        if (_optional(url) case final value?) value,
      if (_optional(legacyUrl) case final value?) value,
    };
    return List<String>.unmodifiable(values);
  }
}
