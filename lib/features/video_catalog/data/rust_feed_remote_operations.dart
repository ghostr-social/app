part of 'rust_feed_remote_source.dart';

extension RustFeedRemoteOperations on RustFeedRemoteSource {
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

  Future<List<VideoPost>> _loadMore(
    FfiFeedSpec spec,
    NostrPublicKeyHex? viewer,
  ) async {
    final session = await _opened(spec, viewer);
    try {
      final loaded = session.warmPage ?? await session.firstPage();
      return _mapped((await session.olderPage(loaded)).posts, null);
    } on Object catch (error, stackTrace) {
      await _sessions.retire(session);
      throw _failure(error, stackTrace);
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

  Future<RustFeedPage> _page(RustFeedSession session, BigInt? cursor) async {
    final warm = session.warmPage;
    if (warm != null && _reaches(warm, cursor)) return warm;
    final loaded = warm ?? await session.firstPage();
    return cursor == null ? loaded : session.olderPage(loaded);
  }

  bool _reaches(RustFeedPage warm, BigInt? cursor) {
    return cursor == null ||
        warm.posts.any((post) => post.feedSortAt <= cursor);
  }

  List<VideoPost> _mapped(List<FfiFeedPost> rows, BigInt? cursor) {
    return _rows.map(rows, cursor);
  }

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
