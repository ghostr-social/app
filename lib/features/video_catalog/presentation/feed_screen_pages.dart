part of 'feed_screen.dart';

extension _FeedScreenPages on _FeedScreenState {
  Widget _feedPages(BuildContext context, FeedLoaded state) {
    final warmPreviousDepth = _warmPreviousDepth(state);
    final playbackIds = _pagePlaybackIds(state, warmPreviousDepth);
    _pagePlayback.synchronize(
      playbackIds: playbackIds,
      keepAliveIds: playbackIds,
    );
    return Stack(
      fit: StackFit.expand,
      children: [
        FeedPageView(
          key: ValueKey(state.kind),
          model: FeedPageModel(
            keys: state.posts.map((post) => ValueKey(post.id.value)),
            rosterRevision: state.rosterRevision,
            activePage: state.activeIndex,
          ),
          onPageChanged: context.read<FeedCubit>().pageChanged,
          itemBuilder: (_, index) =>
              _feedPage(context, state, index, warmPreviousDepth),
        ),
        ..._reserveHosts(state),
      ],
    );
  }

  Iterable<Widget> _reserveHosts(FeedLoaded state) sync* {
    if (!_isVisible || _memoryConstrained) return;
    for (
      var index = state.activeIndex + _pageHostedFutureDepth + 1;
      index < state.posts.length;
      index++
    ) {
      final post = state.posts[index];
      final prepared = state.preparation.forMedia(post.media);
      if (prepared == null) continue;
      yield Positioned.fill(
        key: ValueKey('warp-reserve-${post.id.value}'),
        child: Offstage(child: _reserveSurface(post, prepared)),
      );
    }
  }

  Widget _reserveSurface(VideoPost post, PreparedProgressivePlayback prepared) {
    return widget.bindings.playbackPort.buildSurface(
      PreparedProgressiveVideoPlaybackRequest(
        request: VideoPlaybackSurfaceRequest(
          media: post.media,
          videoId: PlaybackVideoId.parse(post.id),
          isActive: false,
          surfaceScope: _playbackSurfaceScope,
        ),
        prepared: prepared,
      ),
    );
  }

  Widget _feedPage(
    BuildContext context,
    FeedLoaded state,
    int index,
    int warmPreviousDepth,
  ) {
    final post = state.posts[index];
    final source = _playbackSource(state, index, warmPreviousDepth);
    if (source == null) {
      return ColoredBox(key: ValueKey(post.id.value), color: Colors.black);
    }
    return _PlaybackFeedPage(
      controller: _pagePlayback,
      postId: post.id,
      child: _hlsBoundFeedCard(context, state, index, source),
    );
  }

  FeedCardPlaybackSource? _playbackSource(
    FeedLoaded state,
    int index,
    int warmPreviousDepth,
  ) {
    final prepared = _preparedPlayback(state, index);
    if (prepared != null) return FeedCardPlaybackSource.prepared(prepared);
    final current = index == state.activeIndex;
    final previousDistance = state.activeIndex - index;
    if (current || _keepsWarmPrevious(previousDistance, warmPreviousDepth)) {
      return FeedCardPlaybackSource.direct(state.posts[index].media);
    }
    return null;
  }

  bool _keepsWarmPrevious(int distance, int warmPreviousDepth) {
    return distance > 0 && distance <= warmPreviousDepth;
  }

  Set<VideoPostId> _pagePlaybackIds(FeedLoaded state, int warmPreviousDepth) {
    return {
      for (var index = 0; index < state.posts.length; index++)
        if (_playbackSource(state, index, warmPreviousDepth) != null)
          state.posts[index].id,
    };
  }

  int _warmPreviousDepth(FeedLoaded state) {
    if (!_isVisible || _memoryConstrained) return 0;
    final demand = _futureRetentionDemand(state);
    return _playerRetention.warmPreviousDepth(
      preparedFutureCount: demand.prepared,
      canReplenish: demand.canReplenish,
    );
  }

  ({int prepared, bool canReplenish}) _futureRetentionDemand(FeedLoaded state) {
    var prepared = 0;
    var canReplenish = false;
    for (
      var index = state.activeIndex + 1;
      index < state.posts.length;
      index++
    ) {
      final media = state.posts[index].media;
      if (state.preparation.forMedia(media) == null) {
        canReplenish = true;
      } else {
        prepared++;
      }
    }
    return (prepared: prepared, canReplenish: canReplenish);
  }

  PreparedProgressivePlayback? _preparedPlayback(FeedLoaded state, int index) {
    if (index == state.activeIndex) return state.preparation.current;
    final distance = index - state.activeIndex;
    if (!_isVisible || distance < 1 || distance > _pageHostedFutureDepth) {
      return null;
    }
    return state.preparation.forMedia(state.posts[index].media);
  }

  int get _pageHostedFutureDepth {
    return _memoryConstrained ? 1 : _preparedSwipePageDepth;
  }

  Widget _feedCard(
    BuildContext context,
    FeedLoaded state,
    int index,
    FeedCardPlayback playback,
  ) {
    final post = state.posts[index];
    return BlocBuilder<VideoShareCubit, VideoShareState>(
      builder: (context, sharing) => FeedCard(
        post: post,
        playback: playback,
        actions: _actions(context, state, post, sharing),
      ),
    );
  }
}

const _preparedSwipePageDepth = 3;
const _playerRetention = FeedPlayerRetention(
  maximumControllers: warpMaximumConcurrentPlaybackControllers,
  minimumPrevious: 2,
  history: FeedNavigationHistory.ordinary,
);
