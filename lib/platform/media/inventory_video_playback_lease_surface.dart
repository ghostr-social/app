part of 'inventory_video_playback_port.dart';

extension on _InventoryVideoSurfaceState {
  Widget _buildPlaybackSurface(VideoMediaSource media) {
    final lease = _lease;
    if (lease == null) return _buildUnleasedSurface(media);
    _renderedLeases.add(lease);
    return KeyedSubtree(
      key: ObjectKey(lease),
      child: widget.port._delegate.buildSurface(
        media: media,
        isActive: widget.isActive,
        onPlaybackMediaReleased: () => _releaseRenderedLease(lease),
      ),
    );
  }

  Widget _buildUnleasedSurface(VideoMediaSource media) {
    return widget.port._delegate.buildSurface(
      media: media,
      isActive: widget.isActive,
      onPlaybackMediaReleased: widget.onPlaybackMediaReleased,
    );
  }

  void _retireLease() {
    final lease = _lease;
    _lease = null;
    if (lease != null && !_renderedLeases.contains(lease)) lease.release();
  }

  void _releaseRenderedLease(VideoCacheLease lease) {
    if (_renderedLeases.remove(lease)) lease.release();
  }
}
