part of 'video_player_playback_port.dart';

extension _VideoPlayerSurfaceKeys on _VideoPlayerSurfaceDependencies {
  Key surfaceKey(VideoPlaybackSurfaceRequest request) {
    final slot = _exactProgressiveSurfaceSlot(request);
    if (slot == null) {
      return ValueKey((
        this,
        request.media.inventoryPlaybackIdentity,
        request.videoId,
      ));
    }
    return _exactSurfaceKeys.putIfAbsent(
      slot,
      () => GlobalKey<_VideoPlayerSurfaceState>(
        debugLabel: 'warp-${request.videoId?.value ?? 'progressive'}',
      ),
    );
  }

  void releaseSurfaceKey(VideoPlaybackSurfaceRequest request, Key? key) {
    final slot = _exactProgressiveSurfaceSlot(request);
    if (slot == null) return;
    if (identical(_exactSurfaceKeys[slot], key)) {
      _exactSurfaceKeys.remove(slot);
    }
  }
}

typedef _ExactProgressiveSurfaceSlot = (
  VideoMediaCacheIdentity,
  PlaybackVideoId?,
);

_ExactProgressiveSurfaceSlot? _exactProgressiveSurfaceSlot(
  VideoPlaybackSurfaceRequest request,
) {
  if (request.media is! ProxiedProgressiveVideoMediaSource) return null;
  return (request.media.inventoryPlaybackIdentity, request.videoId);
}
