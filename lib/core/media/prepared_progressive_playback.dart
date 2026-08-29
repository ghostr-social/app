import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';

enum PreparedPlaybackReadiness {
  preparing,
  structuralStartable,
  playerVerified;

  bool get isStructurallyStartable => this != preparing;
  bool get isPlayerVerified => this == playerVerified;
}

final class PreparedProgressivePlaybackBinding {
  const PreparedProgressivePlaybackBinding({
    required this.origin,
    required this.sourceRepresentationId,
  });

  final VideoMediaSource origin;
  final VideoRepresentationId sourceRepresentationId;
}

/// One exact loopback asset paired with the canonical source that can renew it.
final class PreparedProgressivePlayback {
  factory PreparedProgressivePlayback.bind({
    required PreparedProgressivePlaybackBinding binding,
    required ProxiedProgressiveVideoMediaSource media,
    required PlaybackAssetAuthority authority,
    required PreparedPlaybackReadiness readiness,
  }) {
    _validateOrigin(binding.origin, authority, binding.sourceRepresentationId);
    _validateProxy(media, authority);
    return PreparedProgressivePlayback._(binding, media, authority, readiness);
  }

  PreparedProgressivePlayback._(
    PreparedProgressivePlaybackBinding binding,
    this.media,
    this.authority,
    this.readiness,
  ) : origin = binding.origin,
      sourceRepresentationId = binding.sourceRepresentationId;

  final VideoMediaSource origin;
  final ProxiedProgressiveVideoMediaSource media;
  final PlaybackAssetAuthority authority;
  final VideoRepresentationId sourceRepresentationId;
  final PreparedPlaybackReadiness readiness;

  bool get isStructurallyStartable => readiness.isStructurallyStartable;

  bool matches(VideoMediaSource candidate) {
    return candidate.inventoryPlaybackIdentity ==
            origin.inventoryPlaybackIdentity &&
        _matchesSource(candidate, authority.deliveryId, sourceRepresentationId);
  }
}

void _validateOrigin(
  VideoMediaSource origin,
  PlaybackAssetAuthority authority,
  VideoRepresentationId sourceRepresentationId,
) {
  if (!origin.canCacheAsSingleFile ||
      !_matchesSource(origin, authority.deliveryId, sourceRepresentationId)) {
    throw ArgumentError.value(origin, 'origin', 'Must match the authority.');
  }
}

bool _matchesSource(
  VideoMediaSource media,
  PlaybackDeliveryId deliveryId,
  VideoRepresentationId representationId,
) {
  try {
    return media.remoteDelivery == VideoMediaDelivery.progressive &&
        media.playbackDeliveryId == deliveryId &&
        VideoRepresentationId.forMedia(media) == representationId;
  } on ArgumentError {
    return false;
  }
}

void _validateProxy(
  ProxiedProgressiveVideoMediaSource media,
  PlaybackAssetAuthority authority,
) {
  if (media.playbackDeliveryId != authority.deliveryId ||
      media.playbackAssetId != authority.assetId) {
    throw ArgumentError.value(media, 'media', 'Must match the authority.');
  }
}
