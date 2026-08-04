import 'dart:async';
import 'dart:developer';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_port.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_post_mapper.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_spec_builder.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_update_queue.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

/// How long one pull waits for the Rust feed to publish a page before
/// it serves whatever the engine has (a feed stays open for its whole
/// life, so "no revision yet" must never hang the caller).
const _pageDeadline = Duration(seconds: 6);

/// ndk parity: NdkNostrVideoEventQuery surfaces every transport
/// problem as this one failure.
const _feedFailure = AppFailure('Could not load Nostr videos.');

/// Serves the pull-shaped [RemoteVideoSource] the app already speaks
/// from the Rust engine's push-shaped feeds (plan §5): open the feed
/// named by the request, take the newest snapshot it publishes, and
/// close it again. Nothing is cached here — the engine owns the store.
final class RustFeedRemoteSource implements RemoteVideoSource {
  const RustFeedRemoteSource({
    required RustFeedPort port,
    String? viewerPubkeyHex,
    RustFeedPostMapper mapper = const RustFeedPostMapper(),
    Duration deadline = _pageDeadline,
  })  : _port = port,
        _viewerPubkeyHex = viewerPubkeyHex,
        _mapper = mapper,
        _deadline = deadline;

  final RustFeedPort _port;
  final String? _viewerPubkeyHex;
  final RustFeedPostMapper _mapper;
  final Duration _deadline;

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
    DateTime? olderThan,
  }) {
    final spec = buildRustFeedSpec(
      creatorIds: creatorIds,
      searchQuery: searchQuery,
      hashtags: hashtags,
      viewerPubkeyHex: _viewerPubkeyHex,
    );
    if (spec == null) return Future.value(const <VideoPost>[]);
    return _load(spec, olderThan);
  }

  Future<List<VideoPost>> _load(FfiFeedSpec spec, DateTime? olderThan) async {
    final feedId = await _opened(spec);
    final queue = RustFeedUpdateQueue(_port.feedUpdates(feedId));
    try {
      return _mapped(await _page(feedId, queue, olderThan), olderThan);
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw _translated(error, stackTrace);
    } finally {
      // Fire-and-forget: a feed whose stream never ends must not hold
      // the close behind its own cancellation.
      unawaited(queue.dispose());
      await _port.closeFeed(feedId);
    }
  }

  Future<String> _opened(FfiFeedSpec spec) async {
    try {
      return await _port.openFeed(spec);
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw _translated(error, stackTrace);
    }
  }

  /// The rows one request claims: the feed's first page, plus one
  /// older page when the caller paginates.
  Future<List<FfiFeedPost>> _page(
    String feedId,
    RustFeedUpdateQueue queue,
    DateTime? olderThan,
  ) async {
    final first = await _firstPage(queue);
    if (olderThan == null) return first;
    final cursor = _seconds(olderThan);
    final more = await _port.loadMore(feedId, olderThanSecs: cursor);
    if (!more) return first;
    return _olderPage(queue, first, cursor);
  }

  /// The first snapshot that carries rows; empty once the feed ends or
  /// the deadline passes without one.
  Future<List<FfiFeedPost>> _firstPage(RustFeedUpdateQueue queue) async {
    while (true) {
      final update = await queue.next(_deadline);
      if (update == null) return const <FfiFeedPost>[];
      if (update.posts.isNotEmpty) return update.posts;
    }
  }

  /// Snapshots are full lists, so an older page shows up as a later
  /// revision that reaches past the cursor.
  Future<List<FfiFeedPost>> _olderPage(
    RustFeedUpdateQueue queue,
    List<FfiFeedPost> loaded,
    BigInt cursor,
  ) async {
    var newest = loaded;
    while (true) {
      final update = await queue.next(_deadline);
      if (update == null) return newest;
      newest = update.posts;
      if (newest.any((post) => post.createdAt < cursor)) return newest;
    }
  }

  /// ndk parity: an older page is the `until:` slice alone, and a
  /// malformed row is skipped instead of sinking the page
  /// (ndk_video_remote_source.dart).
  List<VideoPost> _mapped(List<FfiFeedPost> rows, DateTime? olderThan) {
    final cursor = olderThan == null ? null : _seconds(olderThan);
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

  AppFailure _translated(Object error, StackTrace stackTrace) {
    log(
      'The Rust feed could not be read.',
      name: 'ghostr.video.rustfeed',
      error: error,
      stackTrace: stackTrace,
    );
    return _feedFailure;
  }

  BigInt _seconds(DateTime moment) {
    return BigInt.from(moment.millisecondsSinceEpoch ~/ 1000);
  }
}
