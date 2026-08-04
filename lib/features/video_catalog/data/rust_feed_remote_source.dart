import 'dart:developer';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_page_reader.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_port.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_post_mapper.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_session.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_sessions.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_spec_builder.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

/// Who is signed in right now, or null while signed out. Read once per
/// request and never captured: the app graph is composed before any
/// session is restored and outlives every sign-out, so a captured key
/// would serve the previous account's feed forever.
typedef RustFeedViewer = NostrPublicKeyHex? Function();

/// The viewer of a build that has no session of its own.
NostrPublicKeyHex? noSignedInViewer() => null;

/// Serves the pull-shaped [RemoteVideoSource] the app already speaks
/// from the Rust engine's push-shaped feeds (plan §5): the feed named
/// by the request is opened once and kept open for the life of this
/// source, and each pull takes the snapshot it holds now.
///
/// Nothing is cached here — the engine owns the store, and it keeps
/// filing pages into an open feed all session. That is what makes a
/// returning pull instant instead of another cold relay round trip,
/// and why what it answers only ever grows (see [RustFeedSessions] for
/// the bounds: how many feeds stay open, and `FEED_POST_RETENTION`
/// for how many rows each keeps).
final class RustFeedRemoteSource implements RemoteVideoSource {
  RustFeedRemoteSource({
    required RustFeedPort port,
    RustFeedViewer viewer = noSignedInViewer,
    RustFeedPostMapper mapper = const RustFeedPostMapper(),
    Duration deadline = rustFeedPageDeadline,
  })  : _sessions = RustFeedSessions(port: port, deadline: deadline),
        _viewer = viewer,
        _mapper = mapper;

  final RustFeedSessions _sessions;
  final RustFeedViewer _viewer;
  final RustFeedPostMapper _mapper;

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
    DateTime? olderThan,
  }) {
    final viewer = _viewer()?.value;
    final spec = buildRustFeedSpec(
      creatorIds: creatorIds,
      searchQuery: searchQuery,
      hashtags: hashtags,
      viewerPubkeyHex: viewer,
    );
    if (spec == null) return Future.value(const <VideoPost>[]);
    return _load(spec, viewer, _cursor(olderThan));
  }

  Future<List<VideoPost>> _load(
    FfiFeedSpec spec,
    String? viewer,
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

  Future<RustFeedSession> _opened(FfiFeedSpec spec, String? viewer) async {
    try {
      return await _sessions.open(spec, viewer);
    } on Object catch (error, stackTrace) {
      throw _failure(error, stackTrace);
    }
  }

  /// The rows one request claims. A live feed answers from the
  /// snapshot it already holds and asks for another page behind the
  /// answer; only a feed that never settled a page, or a cursor past
  /// everything it holds, waits for relays.
  Future<RustFeedPage> _page(RustFeedSession session, BigInt? cursor) async {
    final warm = session.warmPage;
    if (warm != null && _reaches(warm, cursor)) {
      session.deepen();
      return warm;
    }
    final loaded = warm ?? await session.firstPage();
    return cursor == null ? loaded : session.olderPage(loaded, cursor);
  }

  /// Whether a warm snapshot answers this request as it stands: a
  /// fresh pull takes it whole, a paginating one needs rows past the
  /// cursor.
  bool _reaches(RustFeedPage warm, BigInt? cursor) {
    return cursor == null ||
        warm.posts.any((post) => post.createdAt <= cursor);
  }

  /// ndk parity: an older page is the `until:` slice alone, and a
  /// malformed row is skipped instead of sinking the page
  /// (ndk_video_remote_source.dart).
  List<VideoPost> _mapped(List<FfiFeedPost> rows, BigInt? cursor) {
    final posts = <VideoPost>[];
    for (final row in rows) {
      if (cursor != null && row.createdAt > cursor) continue;
      _addMapped(posts, row);
    }
    return List<VideoPost>.unmodifiable(posts);
  }

  void _addMapped(List<VideoPost> posts, FfiFeedPost row) {
    try {
      posts.add(_mapper.map(row));
    } on AppFailure catch (failure, stackTrace) {
      log(
        'Skipping a malformed Rust feed row.',
        name: 'ghostr.video.rustfeed',
        error: failure,
        stackTrace: stackTrace,
      );
    }
  }

  /// ndk parity: NdkNostrVideoEventQuery surfaces every transport
  /// problem as the one shared failure, which the page reader already
  /// raises for a failed page.
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
}
