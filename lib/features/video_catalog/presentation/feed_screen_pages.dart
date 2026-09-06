part of 'feed_screen.dart';

extension _FeedScreenPages on _FeedScreenState {
  Widget _feedPages(BuildContext context, FeedLoaded state) {
    _playbackPreview.synchronize(state.kind);
    final playbackIds = _hostedPlaybackIds(state);
    _pagePlayback.synchronize(
      playbackIds: playbackIds,
      keepAliveIds: playbackIds,
    );
    return _pageView(context, state);
  }

  Set<VideoPostId> _hostedPlaybackIds(FeedLoaded state) {
    final warm = _warmPageIndex(state);
    return {
      for (final index in [state.activeIndex, if (warm != null) warm])
        if (_playbackSource(state, index) != null) state.posts[index].id,
    };
  }

  Widget _pageView(BuildContext context, FeedLoaded state) {
    return NotificationListener<ScrollNotification>(
      onNotification: (notification) => _previewScroll(notification, state),
      child: FeedPageView(
        key: ValueKey(state.kind),
        model: FeedPageModel(
          keys: state.posts.map((post) => ValueKey(post.id.value)),
          rosterRevision: state.rosterRevision,
          activePage: state.activeIndex,
        ),
        onPageChanged: (index) => _commitPageNavigation(state, index),
        itemBuilder: (_, index) => _feedPage(context, state, index),
        overlay: _committedOverlay(context, state),
      ),
    );
  }

  Widget _committedOverlay(BuildContext context, FeedLoaded state) {
    final post = state.roster.active;
    return BlocBuilder<VideoShareCubit, VideoShareState>(
      builder: (context, sharing) => FeedCardOverlay(
        key: ValueKey(post.id),
        post: post,
        actions: _actions(context, state, post, sharing),
      ),
    );
  }

  Widget _feedPage(BuildContext context, FeedLoaded state, int index) {
    final post = state.posts[index];
    final source = _playbackSource(state, index);
    if (source == null) {
      return ColoredBox(key: ValueKey(post.id.value), color: Colors.black);
    }
    return _PlaybackFeedPage(
      controller: _pagePlayback,
      postId: post.id,
      child: _hlsBoundFeedCard(context, state, index, source),
    );
  }

  FeedCardPlaybackSource? _playbackSource(FeedLoaded state, int index) {
    final current = index == state.activeIndex;
    if (!current && index != _warmPageIndex(state)) {
      return null;
    }
    final media = state.posts[index].media;
    final prepared = current
        ? state.preparation.current
        : state.preparation.forMedia(media);
    if (prepared != null) return FeedCardPlaybackSource.prepared(prepared);
    if (current || state.hlsAuthorityFor(media) != null) {
      return FeedCardPlaybackSource.direct(media);
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
        showOverlay: false,
      ),
    );
  }
}
