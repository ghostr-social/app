part of 'feed_cubit.dart';

extension FeedCubitUpdateLoading on FeedCubit {
  Future<void> _reloadFromFeedUpdate(
    int feed,
    FeedKind kind,
    bool allowEmpty,
  ) async {
    final result = await _loadFeedUpdate(kind);
    final accepted = _acceptedFeedUpdate(feed, kind, allowEmpty, result);
    if (accepted == null) return;
    _applyFeedUpdate(kind, accepted.posts);
  }

  Future<FeedFetch?> _loadFeedUpdate(FeedKind kind) {
    final loaded = state is FeedLoaded;
    return _loads.newest(
      () => loaded ? _fetch.resync(kind) : _fetch.unwatched(kind),
    );
  }

  FeedFetched? _acceptedFeedUpdate(
    int feed,
    FeedKind kind,
    bool allowEmpty,
    FeedFetch? result,
  ) {
    if (!_acceptsFeedReconciliation(feed, kind)) return null;
    if (result is! FeedFetched) return null;
    return result.posts.isNotEmpty || allowEmpty ? result : null;
  }

  void _applyFeedUpdate(FeedKind kind, List<VideoPost> posts) {
    final current = state;
    if (current is FeedLoaded) return _acceptRefresh(current, posts);
    _acceptLoad(kind, posts);
  }
}
