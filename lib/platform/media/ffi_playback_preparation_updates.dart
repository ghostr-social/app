import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation_updates.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';
import 'package:ghostr/src/rust/api/playback_preparation_stream.dart';

typedef RustPreparationWatch = Stream<FfiPlaybackPreparationPlan> Function();

final class FfiPlaybackPreparationUpdates
    implements PlaybackPreparationUpdates {
  const FfiPlaybackPreparationUpdates({
    RustPreparationWatch watch = ffiPlaybackPreparationEvents,
  }) : _watch = watch;

  final RustPreparationWatch _watch;

  @override
  Stream<PlaybackPreparationPlan> watchPreparation() {
    return _watch().map(_plan);
  }
}

PlaybackPreparationPlan _plan(FfiPlaybackPreparationPlan native) {
  final currentId = native.currentDeliveryId;
  return PlaybackPreparationPlan(
    revision: native.revision,
    currentDeliveryId: currentId == null
        ? null
        : PlaybackDeliveryId.parse(currentId),
    current: _asset(native.current),
    next: _asset(native.next),
  );
}

PlaybackPreparationAsset? _asset(FfiPlaybackPreparationAsset? native) {
  if (native == null) return null;
  final media = ProxiedProgressiveVideoMediaSource(native.playbackUrl);
  return PlaybackPreparationAsset(
    authority: PlaybackAssetAuthority(
      deliveryId: PlaybackDeliveryId.parse(native.deliveryId),
      representationId: VideoRepresentationId.parse(native.representationId),
      assetId: PlaybackAssetId.parse(native.assetId),
    ),
    media: media,
    readiness: switch (native.readiness) {
      FfiPlaybackPreparationReadiness.preparing =>
        PlaybackPreparationReadiness.preparing,
      FfiPlaybackPreparationReadiness.structuralStartable =>
        PlaybackPreparationReadiness.structuralStartable,
    },
  );
}
