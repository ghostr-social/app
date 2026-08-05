import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

enum RemoteVideoPhase { loading, settled, failed }

/// One authoritative, monotonically revisioned snapshot from an open feed.
final class RemoteVideoSnapshot {
  factory RemoteVideoSnapshot({
    required BigInt revision,
    required RemoteVideoPhase phase,
    required List<VideoPost> posts,
  }) {
    return RemoteVideoSnapshot._(
      revision,
      phase,
      List<VideoPost>.unmodifiable(posts),
    );
  }

  const RemoteVideoSnapshot._(this.revision, this.phase, this.posts);

  final BigInt revision;
  final RemoteVideoPhase phase;
  final List<VideoPost> posts;
}

/// Passive domain snapshots from a feed Rust already owns.
abstract interface class RemoteVideoUpdates {
  Stream<RemoteVideoSnapshot> watchRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  });
}
