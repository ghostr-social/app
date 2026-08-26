import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

PlaybackPreparationAsset readyPlaybackPreparation(VideoMediaSource media) {
  const capability = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
  final deliveryId = media.playbackDeliveryId!;
  return PlaybackPreparationAsset(
    authority: PlaybackAssetAuthority(
      deliveryId: deliveryId,
      representationId: VideoRepresentationId.forMedia(media),
      assetId: PlaybackAssetId.parse(capability),
    ),
    media: ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:4040/video.mp4?id=${deliveryId.value}'
      '&cap=$capability',
    ),
    readiness: PlaybackPreparationReadiness.ready,
  );
}
