import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_representation_id.dart';

const testPlaybackCapability = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';

HlsPlaybackAuthority testHlsPlaybackAuthority({String postId = 'delivery'}) =>
    HlsPlaybackAuthority(
      deliveryId: PlaybackDeliveryId.parse(postId),
      representationId: VideoRepresentationId.parse('1' * 64),
      assetRevision: HlsPlaybackAssetRevision.parse(BigInt.one),
    );

PlaybackAssetAuthority testPlaybackAuthority({String postId = 'post-1'}) {
  return PlaybackAssetAuthority(
    deliveryId: PlaybackDeliveryId.parse(postId),
    representationId: VideoRepresentationId.parse(
      '1111111111111111111111111111111111111111111111111111111111111111',
    ),
    assetId: PlaybackAssetId.parse(testPlaybackCapability),
  );
}
