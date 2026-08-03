import 'package:ghostr/core/work/retrieval_scheduler.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

/// Names the screen-level scope a remote video request serves.
String remoteVideoRetrievalContext({
  Set<ProfileId>? creatorIds,
  String? searchQuery,
  Set<String>? hashtags,
}) {
  if (searchQuery != null) return 'search:${searchQuery.trim().toLowerCase()}';
  if (hashtags != null && hashtags.isNotEmpty) {
    final tags = hashtags.map((tag) => tag.toLowerCase()).toList()..sort();
    return 'tag:${tags.join('+')}';
  }
  if (creatorIds != null && creatorIds.isNotEmpty) {
    final ids = creatorIds.map((id) => id as String).toList()..sort();
    return 'profile:${ids.join('+')}';
  }
  return 'feed';
}

/// Routes every remote video load through the shared retrieval queue.
///
/// Each load focuses the scheduler on its own context, so switching search,
/// tag, feed, or profile reorders any queued work in the viewer's favor.
final class ScheduledRemoteVideoSource implements RemoteVideoSource {
  const ScheduledRemoteVideoSource({
    required RemoteVideoSource source,
    required RetrievalScheduler scheduler,
  })  : _source = source,
        _scheduler = scheduler;

  final RemoteVideoSource _source;
  final RetrievalScheduler _scheduler;

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
    DateTime? olderThan,
  }) {
    final context = remoteVideoRetrievalContext(
      creatorIds: creatorIds,
      searchQuery: searchQuery,
      hashtags: hashtags,
    );
    _scheduler.focus(context);
    return _scheduler.run(
      RetrievalRequest(context: context),
      () => _source.loadRemoteFeed(
        creatorIds: creatorIds,
        searchQuery: searchQuery,
        hashtags: hashtags,
        olderThan: olderThan,
      ),
    );
  }
}
