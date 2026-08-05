import 'package:ghostr/core/async/parallel_wait.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/creator_search_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_policy.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_updates.dart';

/// Relay-backed search over the lean source: no local merge, no interaction
/// hydration, no prefetching — results render fast and page like a feed.
class DiscoveryVideoSearchRepository
    implements VideoSearchRepository, VideoSearchUpdates {
  const DiscoveryVideoSearchRepository({
    required RemoteVideoSource videos,
    required CreatorSearchSource creators,
    required SocialGraphRepository social,
    required FailureReporter failureReporter,
  }) : _videos = videos,
       _creators = creators,
       _social = social,
       _failureReporter = failureReporter;

  static const _policy = VideoSearchPolicy();

  final RemoteVideoSource _videos;
  final CreatorSearchSource _creators;
  final SocialGraphRepository _social;
  final FailureReporter _failureReporter;

  @override
  Future<VideoFeedPage> searchVideos(
    String query, {
    DateTime? olderThan,
  }) async {
    if (olderThan != null) return loadMoreVideos(query);
    final normalized = _policy.normalize(query);
    if (normalized == null) return VideoFeedPage(posts: const <VideoPost>[]);
    final tag = _policy.hashtag(normalized);
    final fetched = _videos.loadRemoteFeed(
      searchQuery: tag == null ? normalized : null,
      hashtags: tag == null ? null : <String>{tag},
    );
    final blocked = _social.loadBlockedProfiles();
    final (posts, _) = await waitForBoth(fetched, blocked);
    return _pageWithFreshBlocks(posts, tag);
  }

  @override
  Future<VideoFeedPage> loadMoreVideos(String query) async {
    final normalized = _policy.normalize(query);
    if (normalized == null) return VideoFeedPage(posts: const <VideoPost>[]);
    final tag = _policy.hashtag(normalized);
    final fetched = _videos.loadMoreRemoteFeed(
      searchQuery: tag == null ? normalized : null,
      hashtags: tag == null ? null : <String>{tag},
    );
    final blocked = _social.loadBlockedProfiles();
    final (posts, _) = await waitForBoth(fetched, blocked);
    return _pageWithFreshBlocks(posts, tag);
  }

  @override
  Stream<VideoSearchSnapshot> watchVideos(String query) {
    final normalized = _policy.normalize(query);
    if (normalized == null) return const Stream.empty();
    final tag = _policy.hashtag(normalized);
    return _videos
        .watchRemoteFeed(
          searchQuery: tag == null ? normalized : null,
          hashtags: tag == null ? null : <String>{tag},
        )
        .asyncMap((snapshot) => _liveSnapshot(snapshot, tag));
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
    selected.sort(
      (left, right) => right.publishedAt.compareTo(left.publishedAt),
    );
    return List<VideoPost>.unmodifiable(selected);
  }

  Future<VideoSearchSnapshot> _liveSnapshot(
    RemoteVideoSnapshot snapshot,
    String? tag,
  ) async {
    final page = _page(
      snapshot.posts,
      tag,
      await _social.loadBlockedProfiles(),
    );
    return VideoSearchSnapshot(
      revision: snapshot.revision,
      phase: _searchPhase(snapshot.phase),
      page: page,
    );
  }

  VideoSearchPhase _searchPhase(RemoteVideoPhase phase) {
    return switch (phase) {
      RemoteVideoPhase.loading => VideoSearchPhase.loading,
      RemoteVideoPhase.settled => VideoSearchPhase.settled,
      RemoteVideoPhase.failed => VideoSearchPhase.failed,
    };
  }

  VideoFeedPage _page(
    List<VideoPost> fetched,
    String? tag,
    Set<ProfileId> blocked,
  ) {
    return VideoFeedPage(
      posts: _selectPosts(fetched, tag, blocked),
      nextOlderThan: _nextCursor(fetched),
    );
  }

  Future<VideoFeedPage> _pageWithFreshBlocks(
    List<VideoPost> fetched,
    String? tag,
  ) async {
    return _page(fetched, tag, await _social.loadBlockedProfiles());
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
