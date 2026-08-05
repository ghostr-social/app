import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';

abstract interface class VideoSearchRepository {
  /// One page of videos matching a free-text query or `#hashtag`.
  Future<VideoFeedPage> searchVideos(String query, {DateTime? olderThan});

  /// The next page of [query]; the native feed owns its raw-event cursor.
  Future<VideoFeedPage> loadMoreVideos(String query);

  /// Creators matching a free-text query; empty for hashtag queries.
  Future<List<ProfileSummary>> searchCreators(String query);
}
