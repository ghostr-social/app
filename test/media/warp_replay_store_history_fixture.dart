import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';

PlaybackAssetAuthority replayStoreAuthority(
  PlaybackDeliveryId deliveryId,
  String assetSeed,
) {
  return PlaybackAssetAuthority(
    deliveryId: deliveryId,
    representationId: VideoRepresentationId.parse('1'.padLeft(64, '1')),
    assetId: PlaybackAssetId.parse(assetSeed.padLeft(43, assetSeed)),
  );
}

VideoDeliverySnapshot replayStoreSnapshot(
  PlaybackDeliveryId id, {
  required int bytes,
  int? total,
  PlaybackAssetAuthority? authority,
}) {
  return VideoDeliverySnapshot(
    deliveryId: id,
    phase: VideoDeliveryPhase.preparing,
    bytesPresent: BigInt.from(bytes),
    totalBytes: total == null ? null : BigInt.from(total),
    authority: authority,
  );
}

VideoDeliverySnapshot replayStoreFailedSnapshot(
  PlaybackDeliveryId id,
  int total,
) {
  return VideoDeliverySnapshot(
    deliveryId: id,
    phase: VideoDeliveryPhase.failed,
    bytesPresent: BigInt.zero,
    totalBytes: BigInt.from(total),
  );
}
