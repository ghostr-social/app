part of 'hls_video_playback_port.dart';

extension on _HlsVideoPlaybackSurfaceState {
  Widget _buildPlaybackSurface(ProxiedHlsVideoMediaSource media) {
    final lease = _lease!;
    final upstreamRelease = widget.onPlaybackMediaReleased;
    _renderedLeases.add(lease);
    return KeyedSubtree(
      key: ObjectKey(lease),
      child: widget.port._delegate.buildSurface(
        VideoPlaybackSurfaceRequest(
          media: media,
          videoId: widget.videoId,
          isActive: widget.isActive,
          mode: widget.mode,
          onPlaybackMediaReleased: () {
            final released = _releaseRenderedLease(lease);
            if (released) upstreamRelease?.call();
          },
        ),
      ),
    );
  }

  void _retireLease() {
    final lease = _lease;
    _lease = null;
    if (lease != null && !_renderedLeases.contains(lease)) lease.release();
  }

  bool _releaseRenderedLease(HlsPlaybackLease lease) {
    if (!_renderedLeases.remove(lease)) return false;
    lease.release();
    return true;
  }
}
