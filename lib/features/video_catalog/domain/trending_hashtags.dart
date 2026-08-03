import 'package:ghostr/features/video_catalog/domain/video_post.dart';

abstract interface class TrendingHashtagsSource {
  /// The most-used hashtags across recently published videos.
  Future<List<String>> trendingHashtags();
}

/// Ranks hashtags by how many recent posts carry them; ties break
/// alphabetically so the ordering is stable across refreshes.
List<String> rankTrendingHashtags(List<VideoPost> posts, {int limit = 12}) {
  final counts = <String, int>{};
  for (final post in posts) {
    for (final tag in post.hashtags.toSet()) {
      counts[tag] = (counts[tag] ?? 0) + 1;
    }
  }
  final ranked = counts.keys.toList()
    ..sort((left, right) {
      final byCount = counts[right]!.compareTo(counts[left]!);
      return byCount == 0 ? left.compareTo(right) : byCount;
    });
  return List<String>.unmodifiable(ranked.take(limit));
}
