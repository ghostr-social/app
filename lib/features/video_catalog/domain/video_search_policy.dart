import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_hashtags.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class VideoSearchPolicy {
  const VideoSearchPolicy();

  String? normalize(String rawQuery) {
    final normalized = rawQuery.trim().toLowerCase();
    return normalized.isEmpty ? null : normalized;
  }

  String? hashtag(String query) {
    if (!query.startsWith('#')) return null;
    return normalizeHashtag(query);
  }

  List<VideoPost> select(
    List<VideoPost> posts, {
    required String query,
    required Set<ProfileId> blocked,
  }) {
    final normalized = normalize(query);
    if (normalized == null) return const <VideoPost>[];
    return List<VideoPost>.unmodifiable(
      posts.where((post) => _matches(post, normalized, blocked)),
    );
  }

  bool _matches(
    VideoPost post,
    String query,
    Set<ProfileId> blocked,
  ) {
    if (blocked.contains(post.creator.id)) return false;
    final tag = hashtag(query);
    if (tag != null) return post.hashtags.contains(tag);
    final values = [
      post.caption,
      post.songName,
      post.creator.displayName,
      post.creator.handle,
      ...post.hashtags,
    ];
    return values.any((value) => value.toLowerCase().contains(query));
  }
}
