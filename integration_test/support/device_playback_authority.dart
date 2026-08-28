import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';

PlaybackAssetAuthority devicePlaybackFixtureAuthority(
  ProxiedHlsVideoMediaSource media,
) {
  return PlaybackAssetAuthority(
    deliveryId: media.playbackDeliveryId!,
    representationId: VideoRepresentationId.parse(_representationId),
    assetId: PlaybackAssetId.parse(_assetId),
  );
}

const _representationId =
    '1111111111111111111111111111111111111111111111111111111111111111';
const _assetId = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
