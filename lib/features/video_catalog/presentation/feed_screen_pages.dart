part of 'feed_screen.dart';

extension _FeedScreenPages on _FeedScreenState {
  Widget _feedPages(BuildContext context, FeedLoaded state) {
    final playbackIds = {
      for (
        var index = state.activeIndex;
        index <= state.activeIndex + _pageHostedFutureDepth &&
            index < state.posts.length;
        index++
      )
        if (_playbackSource(state, index) != null) state.posts[index].id,
    };
    _pagePlayback.synchronize(
      playbackIds: playbackIds,
      keepAliveIds: playbackIds,
    );
    return FeedPageView(
      key: ValueKey(state.kind),
      model: FeedPageModel(
        keys: state.posts.map((post) => ValueKey(post.id.value)),
        rosterRevision: state.rosterRevision,
        activePage: state.activeIndex,
      ),
      onPageChanged: context.read<FeedCubit>().pageChanged,
      itemBuilder: (_, index) => _feedPage(context, state, index),
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
    final distance = index - state.activeIndex;
    if (!current && (distance < 1 || distance > _pageHostedFutureDepth)) {
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

  int get _pageHostedFutureDepth => _isVisible && !_memoryConstrained ? 1 : 0;

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
