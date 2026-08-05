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
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

part 'rust_feed_remote_watcher.dart';

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
    implements RemoteVideoSource, RemoteVideoUpdates {
  RustFeedRemoteSource({
    required RustFeedPort port,
    RustFeedViewer viewer = noSignedInViewer,
    RustFeedPostMapper mapper = const RustFeedPostMapper(),
    Duration deadline = rustFeedPageDeadline,
  })  : _sessions = RustFeedSessions(port: port, deadline: deadline),
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
  }) async {
    final viewer = _viewer();
    final spec = _spec(creatorIds, searchQuery, hashtags, viewer);
    if (spec == null) return const <VideoPost>[];
    final session = await _opened(spec, viewer);
    try {
      final loaded = session.warmPage ?? await session.firstPage();
      return _mapped((await session.olderPage(loaded)).posts, null);
    } on Object catch (error, stackTrace) {
      await _sessions.retire(session);
      throw _failure(error, stackTrace);
    }
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

  Future<List<VideoPost>> _load(
    FfiFeedSpec spec,
    NostrPublicKeyHex? viewer,
    BigInt? cursor,
  ) async {
    final session = await _opened(spec, viewer);
    try {
      return _mapped((await _page(session, cursor)).posts, cursor);
    } on Object catch (error, stackTrace) {
      await _sessions.retire(session);
      throw _failure(error, stackTrace);
    } finally {
      await _sessions.retireDead();
    }
  }

  Future<RustFeedSession> _opened(
    FfiFeedSpec spec,
    NostrPublicKeyHex? viewer,
  ) async {
    try {
      return await _sessions.open(spec, viewer);
    } on Object catch (error, stackTrace) {
      throw _failure(error, stackTrace);
    }
  }

  /// The rows one request claims. A live feed answers from the
  /// snapshot it already holds and asks for another page behind the
  /// answer; only a feed that never yielded a page, or a cursor past
  /// everything it holds, waits for relays.
  Future<RustFeedPage> _page(RustFeedSession session, BigInt? cursor) async {
    final warm = session.warmPage;
    if (warm != null && _reaches(warm, cursor)) {
      return warm;
    }
    final loaded = warm ?? await session.firstPage();
    return cursor == null ? loaded : session.olderPage(loaded);
  }

  /// Whether a warm snapshot answers this request as it stands: a
  /// fresh pull takes it whole, a paginating one needs rows past the
  /// cursor.
  bool _reaches(RustFeedPage warm, BigInt? cursor) {
    return cursor == null || warm.posts.any((post) => post.createdAt <= cursor);
  }

  /// An older page is the `until:` slice alone. A malformed row is
  /// skipped instead of sinking the page.
  List<VideoPost> _mapped(List<FfiFeedPost> rows, BigInt? cursor) {
    return _rows.map(rows, cursor);
  }

  /// Every transport problem crosses this boundary as one shared failure.
  AppFailure _failure(Object error, StackTrace stackTrace) {
    if (error is AppFailure) return error;
    log(
      'The Rust feed could not be read.',
      name: 'ghostr.video.rustfeed',
      error: error,
      stackTrace: stackTrace,
    );
    return rustFeedFailure;
  }

  BigInt? _cursor(DateTime? olderThan) {
    if (olderThan == null) return null;
    return BigInt.from(olderThan.millisecondsSinceEpoch ~/ 1000);
  }

  FfiFeedSpec? _spec(
    Set<ProfileId>? creators,
    String? query,
    Set<String>? hashtags,
    NostrPublicKeyHex? viewer,
  ) {
    return buildRustFeedSpec(
      creatorIds: creators,
      searchQuery: query,
      hashtags: hashtags,
      viewerPubkeyHex: viewer?.value,
    );
  }

  RemoteVideoPhase _phase(FfiFeedStage stage) {
    return switch (stage) {
      FfiFeedStage.loading => RemoteVideoPhase.loading,
      FfiFeedStage.settled => RemoteVideoPhase.settled,
      FfiFeedStage.failed => RemoteVideoPhase.failed,
    };
  }
}
