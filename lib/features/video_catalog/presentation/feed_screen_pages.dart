part of 'feed_screen.dart';

extension _FeedScreenPages on _FeedScreenState {
  Widget _feedPages(BuildContext context, FeedLoaded state) {
    return Stack(
      fit: StackFit.expand,
      children: [
        FeedPageView(
          key: ValueKey(state.kind),
          model: FeedPageModel(
            keys: state.posts.map((post) => ValueKey(_playbackId(post))),
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
        key: ValueKey('warp-reserve-${_playbackId(post)}'),
        child: Offstage(child: _reserveSurface(post, prepared)),
      );
    }
  }

  Widget _reserveSurface(VideoPost post, PreparedProgressivePlayback prepared) {
    return widget.bindings.playbackPort.buildSurface(
      PreparedProgressiveVideoPlaybackRequest(
        request: VideoPlaybackSurfaceRequest(
          media: post.media,
          videoId: PlaybackVideoId.parse(_playbackId(post)),
          isActive: false,
          surfaceScope: _playbackSurfaceScope,
        ),
        prepared: prepared,
      ),
    );
  }

  Widget _feedPage(BuildContext context, FeedLoaded state, int index) {
    final post = state.posts[index];
    final prepared = _preparedPlayback(state, index);
    final isCurrent = index == state.activeIndex;
    if (isCurrent) {
      return _feedCard(
        context,
        state,
        index,
        _playback(
          prepared == null
              ? FeedCardPlaybackSource.direct(post.media)
              : FeedCardPlaybackSource.prepared(prepared),
          isCurrent: true,
        ),
      );
    }
    if (prepared != null) {
      return _feedCard(
        context,
        state,
        index,
        _playback(FeedCardPlaybackSource.prepared(prepared), isCurrent: false),
      );
    }
    if (state.preparation.isManaged) {
      return ColoredBox(key: ValueKey(_playbackId(post)), color: Colors.black);
    }
    return _feedCard(
      context,
      state,
      index,
      _playback(FeedCardPlaybackSource.direct(post.media), isCurrent: false),
    );
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
  }) {
    return FeedCardPlayback(
      port: widget.bindings.playbackPort,
      source: source,
      isActive: _isVisible && isCurrent,
      preparedOnly: !isCurrent,
      surfaceScope: _playbackSurfaceScope,
    );
  }

  String _playbackId(VideoPost post) {
    return VideoInteractionTarget.fromPost(post).value;
  }
}
