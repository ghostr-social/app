part of 'gateway_video_playback_port.dart';

final class _GatewayVideoPlaybackSurface extends StatefulWidget {
  const _GatewayVideoPlaybackSurface({
    required this.delegate,
    required this.createCubit,
    required this.media,
    required this.isActive,
    required this.onPlaybackMediaReleased,
  });

  final VideoPlaybackPort delegate;
  final GatewayPlaybackCubit Function(VideoMediaSource) createCubit;
  final VideoMediaSource media;
  final bool isActive;
  final void Function()? onPlaybackMediaReleased;

  @override
  State<_GatewayVideoPlaybackSurface> createState() =>
      _GatewayVideoPlaybackSurfaceState();
}

final class _GatewayVideoPlaybackSurfaceState
    extends State<_GatewayVideoPlaybackSurface> {
  late final GatewayPlaybackCubit _cubit;

  @override
  void initState() {
    super.initState();
    _cubit = widget.createCubit(widget.media);
    unawaited(_cubit.load(widget.media));
  }

  @override
  void didUpdateWidget(covariant _GatewayVideoPlaybackSurface oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.media.inventoryPlaybackIdentity !=
        widget.media.inventoryPlaybackIdentity) {
      unawaited(_cubit.load(widget.media));
    }
  }

  @override
  void dispose() {
    unawaited(_cubit.close());
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<GatewayPlaybackCubit, GatewayPlaybackState>(
      key: ValueKey(widget.media.inventoryPlaybackIdentity),
      bloc: _cubit,
      builder: _buildState,
    );
  }

  Widget _buildState(BuildContext context, GatewayPlaybackState state) {
    return switch (state) {
      GatewayPlaybackFailed() => _buildError(),
      GatewayPlaybackReady(:final media) => _buildPlayback(media),
      GatewayPlaybackPreparing() => _buildLoading(),
    };
  }

  Widget _buildPlayback(ProxiedProgressiveVideoMediaSource media) {
    return widget.delegate.buildSurface(
      media: media,
      isActive: widget.isActive,
      onPlaybackMediaReleased: widget.onPlaybackMediaReleased,
    );
  }

  Widget _buildLoading() {
    final label = widget.isActive ? 'Loading video' : 'Preparing next video';
    return ColoredBox(
      color: AppPalette.videoLoadingBackground,
      child: LoadingPanel(label: label),
    );
  }

  Widget _buildError() {
    return AsyncStatePanel(
      icon: Icons.play_disabled_outlined,
      title: 'Video unavailable',
      message: 'Ghostr could not reach the local video gateway.',
      actionLabel: 'Retry',
      onAction: () => unawaited(_cubit.retry()),
    );
  }
}

final class _UnsupportedStreamPanel extends StatelessWidget {
  const _UnsupportedStreamPanel();

  @override
  Widget build(BuildContext context) {
    return const AsyncStatePanel(
      icon: Icons.play_disabled_outlined,
      title: 'Streaming video unsupported',
      message: 'Secure HLS playback is not available yet.',
    );
  }
}
