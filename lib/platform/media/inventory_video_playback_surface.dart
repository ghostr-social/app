part of 'inventory_video_playback_port.dart';

class _InventoryVideoSurface extends StatefulWidget {
  const _InventoryVideoSurface({
    required this.port,
    required this.media,
    required this.isActive,
    required this.onPlaybackMediaReleased,
  });

  final InventoryVideoPlaybackPort port;
  final VideoMediaSource media;
  final bool isActive;
  final void Function()? onPlaybackMediaReleased;

  @override
  State<_InventoryVideoSurface> createState() => _InventoryVideoSurfaceState();
}

class _InventoryVideoSurfaceState extends State<_InventoryVideoSurface> {
  late VideoMediaSource? _playbackMedia =
      _isDirectPlayback ? widget.media : null;
  VideoMediaSource? _cachedMedia;
  VideoCacheLease? _lease;
  final Set<VideoCacheLease> _renderedLeases = {};
  late bool _isPreparing = !_isDirectPlayback;
  bool _hasPreparationError = false;
  int _requestVersion = 0;

  @override
  void initState() {
    super.initState();
    _requestCache(_priority);
  }

  @override
  void didUpdateWidget(covariant _InventoryVideoSurface oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.media.inventoryPlaybackIdentity !=
        widget.media.inventoryPlaybackIdentity) {
      _resetMedia();
      return;
    }
    if (oldWidget.isActive != widget.isActive) _syncActivity();
  }

  @override
  void dispose() {
    _requestVersion += 1;
    _retireLease();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (_isUnsupported) return _buildUnsupported();
    if (_hasPreparationError) return _buildError();
    final playbackMedia = _playbackMedia;
    if (_isPreparing || playbackMedia == null) return _buildLoading();
    return _buildPlaybackSurface(playbackMedia);
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
      message: 'Ghostr could not safely prepare this video.',
      actionLabel: 'Retry',
      onAction: _retry,
    );
  }

  Widget _buildUnsupported() {
    return const AsyncStatePanel(
      icon: Icons.play_disabled_outlined,
      title: 'Streaming video unsupported',
      message: 'Secure HLS playback is not available yet.',
    );
  }

  bool get _isUnsupported =>
      !_isDirectPlayback && !widget.media.canCacheAsSingleFile;

  bool get _isDirectPlayback =>
      widget.media.isLocal || widget.media is ProxiedHlsVideoMediaSource;

  VideoCachePriority get _priority => widget.isActive
      ? VideoCachePriority.foreground
      : VideoCachePriority.background;

  void _resetMedia() {
    _requestVersion += 1;
    _retireLease();
    _playbackMedia = _isDirectPlayback ? widget.media : null;
    _cachedMedia = null;
    _isPreparing = !_isDirectPlayback;
    _hasPreparationError = false;
    _requestCache(_priority);
  }

  void _syncActivity() {
    if (widget.isActive && _cachedMedia != null) {
      setState(() => _playbackMedia = _cachedMedia);
      return;
    }
    if (widget.isActive && _cachedMedia == null) {
      _requestCache(VideoCachePriority.foreground);
    }
  }

  void _requestCache(VideoCachePriority priority) {
    if (_isDirectPlayback || _isUnsupported) return;
    final version = ++_requestVersion;
    unawaited(_loadCachedMedia(version, priority));
  }

  Future<void> _loadCachedMedia(
    int version,
    VideoCachePriority priority,
  ) async {
    try {
      final lease =
          await widget.port._inventory.acquire(widget.media, priority);
      if (!_isCurrent(version)) {
        lease?.release();
        return;
      }
      _acceptCachedMedia(lease);
    } catch (error, stackTrace) {
      log('Video cache preparation failed.',
          name: 'ghostr.video', error: error, stackTrace: stackTrace);
      if (!_isCurrent(version)) return;
      _rejectCache();
    }
  }

  bool _isCurrent(int version) => mounted && version == _requestVersion;

  void _acceptCachedMedia(VideoCacheLease? lease) {
    if (lease == null) {
      _rejectCache();
      return;
    }
    _retireLease();
    setState(() {
      _lease = lease;
      _isPreparing = false;
      _hasPreparationError = false;
      _cachedMedia = lease.media;
      _playbackMedia = lease.media;
    });
  }

  void _rejectCache() {
    _retireLease();
    setState(() {
      _isPreparing = false;
      _hasPreparationError = true;
      _cachedMedia = null;
      _playbackMedia = null;
    });
  }

  void _retry() {
    _retireLease();
    setState(() {
      _hasPreparationError = false;
      _isPreparing = true;
    });
    _requestCache(VideoCachePriority.foreground);
  }
}
