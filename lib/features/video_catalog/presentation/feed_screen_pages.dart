part of 'feed_screen.dart';

extension _FeedScreenPages on _FeedScreenState {
  Widget _feedPages(BuildContext context, FeedLoaded state) {
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
          itemBuilder: (_, index) => _feedPage(context, state, index),
        ),
        ..._reserveHosts(state),
      ],
    );
  }

  Iterable<Widget> _reserveHosts(FeedLoaded state) sync* {
    if (!_isVisible || _memoryConstrained) return;
    for (
      var index = state.activeIndex + 2;
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

  Widget _feedPage(BuildContext context, FeedLoaded state, int index) {
    final post = state.posts[index];
    final source = _playbackSource(state, index);
    final isCurrent = index == state.activeIndex;
    if (source == null) {
      return ColoredBox(key: ValueKey(post.id.value), color: Colors.black);
    }
    return _PlaybackFeedPage(
      keepAlive: isCurrent || _keepsWarmPrevious(state.activeIndex - index),
      child: _feedCard(
        context,
        state,
        index,
        _playback(
          source,
          isCurrent: isCurrent,
          keepWarmWhenInactive: !isCurrent,
        ),
      ),
    );
  }

  FeedCardPlaybackSource? _playbackSource(FeedLoaded state, int index) {
    final prepared = _preparedPlayback(state, index);
    if (prepared != null) return FeedCardPlaybackSource.prepared(prepared);
    final current = index == state.activeIndex;
    final previousDistance = state.activeIndex - index;
    if (current || _keepsWarmPrevious(previousDistance)) {
      return FeedCardPlaybackSource.direct(state.posts[index].media);
    }
    return null;
  }

  bool _keepsWarmPrevious(int distance) {
    return distance > 0 &&
        distance <= _warmPreviousDepth &&
        _isVisible &&
        !_memoryConstrained;
  }

  PreparedProgressivePlayback? _preparedPlayback(FeedLoaded state, int index) {
    if (index == state.activeIndex) return state.preparation.current;
    if (_isVisible && index == state.activeIndex + 1) {
      return state.preparation.forMedia(state.posts[index].media);
    }
    return null;
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

  FeedCardPlayback _playback(
    FeedCardPlaybackSource source, {
    required bool isCurrent,
    bool keepWarmWhenInactive = false,
  }) {
    return FeedCardPlayback(
      port: widget.bindings.playbackPort,
      source: source,
      isActive: _isVisible && isCurrent,
      preparedOnly: !isCurrent,
      keepWarmWhenInactive: keepWarmWhenInactive,
      surfaceScope: _playbackSurfaceScope,
    );
  }
}

const _warmPreviousDepth = 2;

final class _PlaybackFeedPage extends StatefulWidget {
  const _PlaybackFeedPage({required this.keepAlive, required this.child});

  final bool keepAlive;
  final Widget child;

  @override
  State<_PlaybackFeedPage> createState() => _PlaybackFeedPageState();
}

final class _PlaybackFeedPageState extends State<_PlaybackFeedPage>
    with AutomaticKeepAliveClientMixin {
  @override
  bool get wantKeepAlive => widget.keepAlive;

  @override
  void didUpdateWidget(covariant _PlaybackFeedPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.keepAlive != widget.keepAlive) updateKeepAlive();
  }

  @override
  Widget build(BuildContext context) {
    super.build(context);
    return widget.child;
  }
}
