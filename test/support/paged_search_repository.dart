import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';

/// Serves scripted search pages and records every request.
class PagedSearchRepository implements VideoSearchRepository {
  PagedSearchRepository({
    List<List<VideoPost>> pages = const <List<VideoPost>>[],
    this.creators = const <ProfileSummary>[],
  }) : _pages = List<List<VideoPost>>.of(pages);

  final List<List<VideoPost>> _pages;
  final List<ProfileSummary> creators;
  final List<String> queries = <String>[];
  final List<String> creatorQueries = <String>[];
  final List<DateTime?> olderThans = <DateTime?>[];
  final List<String> loadMoreQueries = <String>[];
  Object? videosFailure;

  @override
  Future<VideoFeedPage> searchVideos(String query,
      {DateTime? olderThan}) async {
    queries.add(query);
    olderThans.add(olderThan);
    return _nextPage();
  }

  @override
  Future<VideoFeedPage> loadMoreVideos(String query) async {
    loadMoreQueries.add(query);
    return _nextPage();
  }

  VideoFeedPage _nextPage() {
    if (videosFailure case final Object failure) throw failure;
    final posts = _pages.isEmpty ? const <VideoPost>[] : _pages.removeAt(0);
    return VideoFeedPage(
      posts: posts,
      nextOlderThan: _pages.isEmpty ? null : _oldest(posts),
    );
  }

  @override
  Future<List<ProfileSummary>> searchCreators(String query) async {
    creatorQueries.add(query);
    return creators;
  }

  DateTime? _oldest(List<VideoPost> posts) {
    if (posts.isEmpty) return null;
    var oldest = posts.first.publishedAt;
    for (final post in posts.skip(1)) {
      if (post.publishedAt.isBefore(oldest)) oldest = post.publishedAt;
    }
    return oldest.subtract(const Duration(seconds: 1));
  }
}
