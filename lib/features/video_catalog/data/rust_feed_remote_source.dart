import 'dart:async';
import 'dart:developer';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_page_reader.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_port.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_post_mapper.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_rows.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_session.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_sessions.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_spec_builder.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/following_feed_scope.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

part 'rust_feed_remote_watcher.dart';
part 'rust_feed_remote_operations.dart';

/// Who is signed in right now, or null while signed out. Read once per
/// request and never captured: the app graph is composed before any
/// session is restored and outlives every sign-out, so a captured key
/// would serve the previous account's feed forever.
typedef RustFeedViewer = NostrPublicKeyHex? Function();

/// The viewer of a build that has no session of its own.
NostrPublicKeyHex? noSignedInViewer() => null;

/// Adapts the Rust engine's push-shaped feed snapshots to the app's
/// pull-shaped [RemoteVideoSource]. The feed named by a request opens
/// once, stays open for this source's lifetime, and each pull takes its
/// current snapshot.
///
/// Nothing is cached here — the engine owns the store, and it keeps
/// filing pages into an open feed all session. That is what makes a
/// returning pull instant instead of another cold relay round trip,
/// while authoritative snapshots can reconcile provisional rows (see
/// [RustFeedSessions] for the bound on how many feeds stay open).
final class RustFeedRemoteSource
    implements
        RemoteVideoSource,
        RemoteVideoUpdates,
        FollowingRemoteVideoSource,
        FollowingRemoteVideoUpdates {
  RustFeedRemoteSource({
    required RustFeedPort port,
    RustFeedViewer viewer = noSignedInViewer,
    RustFeedPostMapper mapper = const RustFeedPostMapper(),
    Duration deadline = rustFeedPageDeadline,
  }) : _sessions = RustFeedSessions(port: port, deadline: deadline),
       _viewer = viewer,
       _rows = RustFeedRows(mapper);

  final RustFeedSessions _sessions;
  final RustFeedViewer _viewer;
  final RustFeedRows _rows;

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
    DateTime? olderThan,
  }) {
    final viewer = _viewer();
    final spec = _spec(creatorIds, searchQuery, hashtags, viewer);
    if (spec == null) return Future.value(const <VideoPost>[]);
    return _load(spec, viewer, _cursor(olderThan));
  }

  @override
  Future<List<VideoPost>> loadMoreRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) {
    final viewer = _viewer();
    final spec = _spec(creatorIds, searchQuery, hashtags, viewer);
    if (spec == null) return Future.value(const <VideoPost>[]);
    return _loadMore(spec, viewer);
  }

  @override
  Future<List<VideoPost>> loadFollowingRemoteFeed(
    FollowingFeedScope scope, {
    DateTime? olderThan,
  }) {
    final spec = buildRustFollowingFeedSpec(
      scope.creators,
      viewerPubkeyHex: scope.viewer.value,
    );
    if (spec == null) return Future.value(const <VideoPost>[]);
    return _load(spec, scope.viewer, _cursor(olderThan));
  }

  @override
  Stream<RemoteVideoSnapshot> watchRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) {
    final viewer = _viewer();
    final spec = _spec(creatorIds, searchQuery, hashtags, viewer);
    if (spec == null) return const Stream.empty();
    return RustFeedRemoteWatcher(this, spec, viewer).stream;
  }

  @override
  Stream<RemoteVideoSnapshot> watchFollowingRemoteFeed(
    FollowingFeedScope scope,
  ) {
    final spec = buildRustFollowingFeedSpec(
      scope.creators,
      viewerPubkeyHex: scope.viewer.value,
    );
    if (spec == null) return settledEmptyRemoteVideoUpdates();
    return RustFeedRemoteWatcher(this, spec, scope.viewer).stream;
  }
}
