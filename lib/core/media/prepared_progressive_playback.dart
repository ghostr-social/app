import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';

/// One exact loopback asset paired with the canonical source that can renew it.
final class PreparedProgressivePlayback {
  factory PreparedProgressivePlayback.bind({
    required VideoMediaSource origin,
    required ProxiedProgressiveVideoMediaSource media,
    required PlaybackAssetAuthority authority,
  }) {
    _validateOrigin(origin, authority);
    _validateProxy(media, authority);
    return PreparedProgressivePlayback._(origin, media, authority);
  }

  const PreparedProgressivePlayback._(this.origin, this.media, this.authority);

  final VideoMediaSource origin;
  final ProxiedProgressiveVideoMediaSource media;
  final PlaybackAssetAuthority authority;

  bool matches(VideoMediaSource candidate) {
    return candidate.inventoryPlaybackIdentity ==
            origin.inventoryPlaybackIdentity &&
        _matchesAuthority(candidate, authority);
  }
}

void _validateOrigin(
  VideoMediaSource origin,
  PlaybackAssetAuthority authority,
) {
  if (!origin.canCacheAsSingleFile || !_matchesAuthority(origin, authority)) {
    throw ArgumentError.value(origin, 'origin', 'Must match the authority.');
  }
}

bool _matchesAuthority(
  VideoMediaSource media,
  PlaybackAssetAuthority authority,
) {
  try {
    return media.remoteDelivery == VideoMediaDelivery.progressive &&
        media.playbackDeliveryId == authority.deliveryId &&
        VideoRepresentationId.forMedia(media) == authority.representationId;
  } on ArgumentError {
    return false;
  }
}

void _validateProxy(
  ProxiedProgressiveVideoMediaSource media,
  PlaybackAssetAuthority authority,
) {
  final query = media.playbackUri.queryParameters;
  if (query['id'] != authority.deliveryId.value ||
      query['cap'] != authority.assetId.value) {
    throw ArgumentError.value(media, 'media', 'Must match the authority.');
  }
}
