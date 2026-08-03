import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/creator_search_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_policy.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';

/// Relay-backed search over the lean source: no local merge, no interaction
/// hydration, no prefetching — results render fast and page like a feed.
class DiscoveryVideoSearchRepository implements VideoSearchRepository {
  const DiscoveryVideoSearchRepository({
    required RemoteVideoSource videos,
    required CreatorSearchSource creators,
    required SocialGraphRepository social,
    required FailureReporter failureReporter,
  })  : _videos = videos,
        _creators = creators,
        _social = social,
        _failureReporter = failureReporter;

  static const _policy = VideoSearchPolicy();

  final RemoteVideoSource _videos;
  final CreatorSearchSource _creators;
  final SocialGraphRepository _social;
  final FailureReporter _failureReporter;

  @override
  Future<VideoFeedPage> searchVideos(String query, {DateTime? olderThan}) async {
    final normalized = _policy.normalize(query);
    if (normalized == null) return VideoFeedPage(posts: const <VideoPost>[]);
    final tag = _policy.hashtag(normalized);
    final fetched = await _videos.loadRemoteFeed(
      searchQuery: tag == null ? normalized : null,
      hashtags: tag == null ? null : <String>{tag},
      olderThan: olderThan,
    );
    final selected =
        _selectPosts(fetched, tag, await _social.loadBlockedProfiles());
    return VideoFeedPage(posts: selected, nextOlderThan: _nextCursor(fetched));
  }

  @override
  Future<List<ProfileSummary>> searchCreators(String query) async {
    final normalized = _policy.normalize(query);
    if (normalized == null || _policy.hashtag(normalized) != null) {
      return const <ProfileSummary>[];
    }
    try {
      final creators = await _creators.searchCreators(normalized);
      final blocked = await _social.loadBlockedProfiles();
      return List<ProfileSummary>.unmodifiable(
        creators.where((creator) => !blocked.contains(creator.id)),
      );
    } on AppFailure catch (error, stackTrace) {
      // Creator rows are additive; a search-relay hiccup must not hide videos.
      _failureReporter.report(
        source: 'DiscoveryVideoSearchRepository.searchCreators',
        error: error,
        stackTrace: stackTrace,
      );
      return const <ProfileSummary>[];
    }
  }

  List<VideoPost> _selectPosts(
    List<VideoPost> fetched,
    String? tag,
    Set<ProfileId> blocked,
  ) {
    final selected = fetched.where((post) {
      if (blocked.contains(post.creator.id)) return false;
      // NIP-50 text matching is the relay's judgement; tags we can recheck.
      return tag == null || post.hashtags.contains(tag);
    }).toList();
    selected.sort((left, right) => right.publishedAt.compareTo(left.publishedAt));
    return List<VideoPost>.unmodifiable(selected);
  }

  // The cursor advances by what was fetched, not what survived filtering,
  // so pages full of blocked creators cannot stall pagination.
  DateTime? _nextCursor(List<VideoPost> fetched) {
    if (fetched.isEmpty) return null;
    var oldest = fetched.first.publishedAt;
    for (final post in fetched.skip(1)) {
      if (post.publishedAt.isBefore(oldest)) oldest = post.publishedAt;
    }
    return oldest.subtract(const Duration(seconds: 1));
  }
}
