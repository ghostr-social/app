part of 'feed_cubit.dart';

extension FeedCubitBackfill on FeedCubit {
  void _ensureBuffered() {
    final current = state;
    if (!_isSurfaceVisible || current is! FeedLoaded) return;
    if (_backfill.isStarved(current.roster)) unawaited(loadMore());
  }

  Future<void> loadMore() async {
    if (_isClosing || !_isSurfaceVisible || state is! FeedLoaded) return;
    for (var dry = 0; dry < _backfill.dryPageLimit; dry += 1) {
      if (!await _digOnce()) return;
    }
    _backfillRetry.schedule(_ensureBuffered);
  }

  Future<bool> _digOnce() async {
    final dug = await _backfill.dig(state.kind);
    return switch (dug) {
      FeedDigFailed(:final failure) => _failedDig(failure),
      FeedDigSkipped(:final retryable) => _skippedDig(retryable),
      FeedDigPage(:final posts, :final hasMore, :final cursorAdvanced) =>
        _acceptedDig(posts, hasMore && cursorAdvanced),
    };
  }

  bool _skippedDig(bool retryable) {
    if (retryable && _isSurfaceVisible) {
      _backfillRetry.schedule(_ensureBuffered);
    }
    return false;
  }

  bool _failedDig(FeedUnavailable failure) {
    _showNotice(feedLoadFailureMessage(failure.failure));
    return false;
  }

  bool _acceptedDig(List<VideoPost> posts, bool hasMore) {
    if (_appendPage(posts)) {
      _backfillRetry.succeeded();
      _ensureBuffered();
      return false;
    }
    if (hasMore) return true;
    _backfillRetry.reset();
    return false;
  }

  bool _appendPage(List<VideoPost> incoming) {
    final current = state;
    if (current is! FeedLoaded) return false;
    final posts = _session.appended(current.roster, incoming);
    if (posts == null) return false;
    emit(current.withPosts(posts));
    unawaited(_settleReposts());
    _viewer.rosterChanged(posts, current.activeIndex);
    return true;
  }
}
