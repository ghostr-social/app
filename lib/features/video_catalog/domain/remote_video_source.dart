import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

abstract interface class RemoteVideoSource implements RemoteVideoUpdates {
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
    DateTime? olderThan,
  });

  /// Claims the next native page. Rust chooses its cursor from raw events.
  Future<List<VideoPost>> loadMoreRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  });
}

class DisabledRemoteVideoSource implements RemoteVideoSource {
  const DisabledRemoteVideoSource(this.message);

  final String message;

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
    DateTime? olderThan,
  }) {
    throw AppFailure(message);
  }

  @override
  Future<List<VideoPost>> loadMoreRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) {
    throw AppFailure(message);
  }

  @override
  Stream<RemoteVideoSnapshot> watchRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) {
    return Stream<RemoteVideoSnapshot>.error(AppFailure(message));
  }
}
