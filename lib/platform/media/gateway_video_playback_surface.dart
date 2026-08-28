part of 'gateway_video_playback_port.dart';

final class _GatewayVideoPlaybackSurface extends StatefulWidget {
  const _GatewayVideoPlaybackSurface({
    required this.delegate,
    required this.gateway,
    required this.createCubit,
    required this.request,
  });

  final VideoPlaybackPort delegate;
  final ProgressivePlaybackGatewayPort gateway;
  final GatewayPlaybackCubit Function() createCubit;
  final VideoPlaybackSurfaceRequest request;

  VideoMediaSource get media => request.media;
  PlaybackVideoId? get videoId => request.videoId;
  bool get isActive => request.isActive;
  VideoPlaybackMode get mode => request.mode;
  VoidCallback? get onPlaybackMediaReleased => request.onPlaybackMediaReleased;
  PreparedProgressivePlayback? get prepared {
    final request = this.request;
    return request is PreparedProgressiveVideoPlaybackRequest
        ? request.prepared
        : null;
  }

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
    _cubit = widget.createCubit();
    unawaited(_load());
  }

  @override
  void didUpdateWidget(covariant _GatewayVideoPlaybackSurface oldWidget) {
    super.didUpdateWidget(oldWidget);
    unawaited(_load());
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
      GatewayPlaybackReady(:final origin, :final media) => _buildPlayback(
        origin,
        media,
      ),
      GatewayPlaybackPreparing() => _buildLoading(),
    };
  }

  Widget _buildPlayback(
    VideoMediaSource origin,
    ProxiedProgressiveVideoMediaSource media,
  ) {
    return widget.delegate.buildSurface(
      VideoPlaybackSurfaceRequest(
        media: media,
        videoId: widget.videoId,
        isActive: widget.isActive,
        mode: widget.mode,
        surfaceScope: widget.request.surfaceScope,
        playbackDeliveryId: widget.request.playbackDeliveryId,
        reservesPreparedDecoder: widget.request.reservesPreparedDecoder,
        keepWarmWhenInactive: widget.request.keepWarmWhenInactive,
        authority: _readyAuthority(origin, media),
        progressiveRefresh: _GatewayProgressivePlaybackRefresh(
          widget.gateway,
          origin,
        ),
        onPlaybackMediaReleased: widget.onPlaybackMediaReleased,
        preview: widget.request.preview,
      ),
    );
  }

  PlaybackAssetAuthority? _readyAuthority(
    VideoMediaSource origin,
    ProxiedProgressiveVideoMediaSource media,
  ) {
    final prepared = widget.prepared;
    if (prepared != null &&
        prepared.matches(origin) &&
        prepared.media.playbackUri == media.playbackUri) {
      return prepared.authority;
    }
    return _resolvedAuthority(origin, media);
  }

  Future<void> _load() {
    return _cubit.load(widget.media, prepared: widget.prepared);
  }

  Widget _buildLoading() {
    final label = widget.isActive ? 'Loading video' : 'Preparing next video';
    return VideoLoadingSurface(label: label, preview: widget.request.preview);
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

PlaybackAssetAuthority? _resolvedAuthority(
  VideoMediaSource origin,
  ProxiedProgressiveVideoMediaSource media,
) {
  if (!_sameResolvedDelivery(origin, media)) return null;
  return _parseResolvedAuthority(origin, media);
}

bool _sameResolvedDelivery(
  VideoMediaSource origin,
  ProxiedProgressiveVideoMediaSource media,
) {
  final originId = origin.playbackDeliveryId;
  return originId != null && originId == media.playbackDeliveryId;
}

PlaybackAssetAuthority? _parseResolvedAuthority(
  VideoMediaSource origin,
  ProxiedProgressiveVideoMediaSource media,
) {
  try {
    final deliveryId = media.playbackDeliveryId;
    return PlaybackAssetAuthority(
      deliveryId: deliveryId!,
      representationId: VideoRepresentationId.forMedia(origin),
      assetId: media.playbackAssetId,
    );
  } on ArgumentError {
    return null;
  } on FormatException {
    return null;
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
