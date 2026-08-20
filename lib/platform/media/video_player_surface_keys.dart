part of 'video_player_playback_port.dart';

extension _VideoPlayerSurfaceKeys on _VideoPlayerSurfaceDependencies {
  Key surfaceKey(VideoPlaybackSurfaceRequest request) {
    final authority = request.authority;
    if (authority == null) {
      return ValueKey((
        this,
        request.media.inventoryPlaybackIdentity,
        request.videoId,
      ));
    }
    return _preparedSurfaceKeys.putIfAbsent(
      authority,
      () => GlobalKey<_VideoPlayerSurfaceState>(
        debugLabel: 'warp-${authority.deliveryId.value}',
      ),
    );
  }

  void releaseSurfaceKey(PlaybackAssetAuthority? authority, Key? key) {
    if (authority == null) return;
    if (identical(_preparedSurfaceKeys[authority], key)) {
      _preparedSurfaceKeys.remove(authority);
    }
  }
}
