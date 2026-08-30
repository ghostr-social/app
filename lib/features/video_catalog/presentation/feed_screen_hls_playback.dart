part of 'feed_screen.dart';

typedef _FeedPlaybackTarget = ({
  FeedLoaded state,
  int index,
  FeedCardPlaybackSource source,
});

extension _FeedScreenHlsPlayback on _FeedScreenState {
  Widget _hlsBoundFeedCard(
    BuildContext context,
    FeedLoaded state,
    int index,
    FeedCardPlaybackSource source,
  ) {
    final media = state.posts[index].media;
    final target = (state: state, index: index, source: source);
    return BlocSelector<FeedCubit, FeedState, HlsPlaybackAuthority?>(
      selector: (current) =>
          current is FeedLoaded ? current.hlsAuthorityFor(media) : null,
      builder: (context, authority) => _feedCard(
        context,
        state,
        index,
        _playback(context, target, authority),
      ),
    );
  }

  FeedCardPlayback _playback(
    BuildContext context,
    _FeedPlaybackTarget target,
    HlsPlaybackAuthority? authority,
  ) {
    final isCurrent = target.index == target.state.activeIndex;
    final cubit = context.read<FeedCubit>();
    return FeedCardPlayback(
      port: widget.bindings.playbackPort,
      source: target.source,
      isActive: _isVisible && isCurrent,
      preparedOnly: !isCurrent,
      keepWarmWhenInactive: !isCurrent,
      surfaceScope: _playbackSurfaceScope,
      hlsAuthority: authority,
      onHlsFirstFrameRendered: cubit.hlsFirstFrameRendered,
      onPlaybackMediaReleased: authority == null
          ? null
          : () => cubit.hlsPlaybackReleased(authority),
    );
  }
}
