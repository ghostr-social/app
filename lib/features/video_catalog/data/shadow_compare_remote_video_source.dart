import 'dart:async';
import 'dart:developer';

import 'package:ghostr/features/video_catalog/data/feed_parity_divergence.dart';
import 'package:ghostr/features/video_catalog/data/scheduled_remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

typedef FeedParityLogger = void Function(String message);

/// One pull request, carried whole so every helper stays inside the
/// four-parameter budget.
typedef _FeedRequest = ({
  Set<ProfileId>? creatorIds,
  String? searchQuery,
  Set<String>? hashtags,
  DateTime? olderThan,
});

/// Where parity findings land while the Rust pipeline is on trial.
void logFeedParity(String message) => log(message, name: 'ghostr.feedparity');

/// Runs both discovery pipelines against the same request and reports
/// where they disagree (plan §5 step 6).
///
/// The primary pipeline is the truth: its list is returned untouched
/// and on its own schedule. The shadow runs alongside it and can only
/// ever produce a log line — its results, its failures, and its
/// latency never reach the caller.
final class ShadowCompareRemoteVideoSource implements RemoteVideoSource {
  const ShadowCompareRemoteVideoSource({
    required RemoteVideoSource primary,
    required RemoteVideoSource shadow,
    FeedParityLogger logger = logFeedParity,
  })  : _primary = primary,
        _shadow = shadow,
        _log = logger;

  final RemoteVideoSource _primary;
  final RemoteVideoSource _shadow;
  final FeedParityLogger _log;

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
    DateTime? olderThan,
  }) async {
    final request = (
      creatorIds: creatorIds,
      searchQuery: searchQuery,
      hashtags: hashtags,
      olderThan: olderThan,
    );
    final shadowed = _shadowed(request);
    final posts = await _load(_primary, request);
    unawaited(shadowed.then((shadow) => _report(request, posts, shadow)));
    return posts;
  }

  /// The shadow's rows, or null when it failed — a pipeline on trial
  /// never breaks the viewer's feed.
  Future<List<VideoPost>?> _shadowed(_FeedRequest request) async {
    try {
      return await _load(_shadow, request);
    } on Object catch (error) {
      _log('Shadow feed failed for ${_context(request)}: $error');
      return null;
    }
  }

  void _report(
    _FeedRequest request,
    List<VideoPost> posts,
    List<VideoPost>? shadow,
  ) {
    if (shadow == null) return;
    final divergence = FeedParityDivergence.between(posts, shadow);
    if (divergence == null) return;
    _log('Feed parity divergence for ${_context(request)}: $divergence');
  }

  Future<List<VideoPost>> _load(
    RemoteVideoSource source,
    _FeedRequest request,
  ) {
    return source.loadRemoteFeed(
      creatorIds: request.creatorIds,
      searchQuery: request.searchQuery,
      hashtags: request.hashtags,
      olderThan: request.olderThan,
    );
  }

  String _context(_FeedRequest request) {
    return remoteVideoRetrievalContext(
      creatorIds: request.creatorIds,
      searchQuery: request.searchQuery,
      hashtags: request.hashtags,
    );
  }
}
