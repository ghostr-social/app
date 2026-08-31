part of 'video_player_playback_port.dart';

extension _VideoPlayerSurfaceKeys on _VideoPlayerSurfaceDependencies {
  Key surfaceKey(VideoPlaybackSurfaceRequest request) {
    final slot = _exactProgressiveSurfaceSlot(request);
    if (slot == null) {
      return ValueKey((
        this,
        request.media.inventoryPlaybackIdentity,
        request.videoId,
        request.playbackDeliveryId,
        request.hlsAuthority,
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
  VideoPlaybackSurfaceScope,
  PlaybackDeliveryId,
  VideoRepresentationId,
  PlaybackVideoId?,
);

_ExactProgressiveSurfaceSlot? _exactProgressiveSurfaceSlot(
  VideoPlaybackSurfaceRequest request,
) {
  final scope = request.surfaceScope;
  final media = request.media;
  final authority = request.authority;
  if (scope == null ||
      media is! ProxiedProgressiveVideoMediaSource ||
      authority == null) {
    return null;
  }
  if (!_matchesExactProgressiveIdentity(request, media, authority)) return null;
  return (
    scope,
    authority.deliveryId,
    authority.representationId,
    request.videoId,
  );
}

bool _matchesExactProgressiveIdentity(
  VideoPlaybackSurfaceRequest request,
  ProxiedProgressiveVideoMediaSource media,
  PlaybackAssetAuthority authority,
) {
  final requested = request.playbackDeliveryId;
  return _proxyMatches(media, authority) &&
      (requested == null || requested == authority.deliveryId);
}
