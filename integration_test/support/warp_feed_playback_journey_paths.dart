part of 'warp_feed_playback_journey.dart';

extension WarpFeedPlaybackJourneyPaths on WarpFeedPlaybackJourney {
  List<String> futureRemotePaths(int count) {
    final paths = allFutureRemotePaths().take(count).toList(growable: false);
    if (paths.length != count) throw RangeError.value(count);
    return paths;
  }

  List<String> allFutureRemotePaths() {
    final state = _loadedState();
    return state.posts
        .skip(state.activeIndex + 1)
        .map((post) => Uri.parse(post.media.remoteUrl!).path)
        .toList(growable: false);
  }

  List<String> remotePathsFor(Iterable<WarpFeedCurrentPreparation> assets) {
    final state = _loadedState();
    return assets
        .map(
          (asset) => state.posts.singleWhere((post) {
            return post.media.playbackDeliveryId == asset.authority.deliveryId;
          }),
        )
        .map((post) => Uri.parse(post.media.remoteUrl!).path)
        .toList(growable: false);
  }

  FeedLoaded _loadedState() {
    final state = cubit.state;
    if (state is! FeedLoaded) throw StateError('Feed is not loaded.');
    return state;
  }
}
