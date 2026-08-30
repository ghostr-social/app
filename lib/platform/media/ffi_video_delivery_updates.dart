import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/src/rust/api/delivery_events_stream.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

typedef RustDeliveryWatch = Stream<FfiDeliveryEvent> Function();

/// App-lifetime translation of Rust playable-prefix events.
final class FfiVideoDeliveryUpdates implements VideoDeliveryUpdates {
  FfiVideoDeliveryUpdates({RustDeliveryWatch watch = ffiDeliveryEvents})
    : _watch = watch;

  final RustDeliveryWatch _watch;
  late final Stream<VideoDeliverySnapshot> _events = _nativeEvents()
      .map(_snapshot)
      .asBroadcastStream();

  @override
  Stream<VideoDeliverySnapshot> watchDelivery() => _events;

  Stream<FfiDeliveryEvent> _nativeEvents() async* {
    await for (final event in _watch()) {
      if (event.kind != FfiDeliveryEventKind.error) {
        yield event;
        continue;
      }
      yield* Stream<FfiDeliveryEvent>.error(
        VideoDeliveryObservationException(event.postId, event.detail),
      );
    }
  }
}

final class VideoDeliveryObservationException implements Exception {
  const VideoDeliveryObservationException(this.postId, this.detail);

  final String postId;
  final String? detail;

  @override
  String toString() => 'Video delivery observation failed for $postId: $detail';
}

VideoDeliverySnapshot _snapshot(FfiDeliveryEvent event) {
  final deliveryId = PlaybackDeliveryId.parse(event.postId);
  return VideoDeliverySnapshot(
    deliveryId: deliveryId,
    phase: _phase(event),
    bytesPresent: event.bytesPresent,
    totalBytes: event.totalBytes,
    eta: event.etaMs == null
        ? null
        : Duration(milliseconds: event.etaMs!.toInt()),
    detail: event.detail,
    authority: _authority(event, deliveryId),
    hlsAuthority: _hlsAuthority(event, deliveryId),
  );
}

HlsPlaybackAuthority? _hlsAuthority(
  FfiDeliveryEvent event,
  PlaybackDeliveryId deliveryId,
) {
  final nativeDelivery = event.hlsDeliveryId;
  final representation = event.hlsRepresentationId;
  final revision = event.hlsAssetRevision;
  if (nativeDelivery == null && representation == null && revision == null) {
    return null;
  }
  if (nativeDelivery == null || representation == null || revision == null) {
    throw const FormatException('Incomplete HLS delivery authority.');
  }
  final authority = HlsPlaybackAuthority(
    deliveryId: PlaybackDeliveryId.parse(nativeDelivery),
    representationId: VideoRepresentationId.parse(representation),
    assetRevision: HlsPlaybackAssetRevision.parse(revision),
  );
  if (authority.deliveryId != deliveryId) {
    throw const FormatException('HLS delivery authority mismatch.');
  }
  return authority;
}

PlaybackAssetAuthority? _authority(
  FfiDeliveryEvent event,
  PlaybackDeliveryId deliveryId,
) {
  final representation = event.representationId;
  final asset = event.assetId;
  if (representation == null && asset == null) return null;
  if (representation == null || asset == null) {
    throw const FormatException('Incomplete delivery authority.');
  }
  return PlaybackAssetAuthority(
    deliveryId: deliveryId,
    representationId: VideoRepresentationId.parse(representation),
    assetId: PlaybackAssetId.parse(asset),
  );
}

VideoDeliveryPhase _phase(FfiDeliveryEvent event) {
  if (event.kind == FfiDeliveryEventKind.failed) {
    return VideoDeliveryPhase.failed;
  }
  return event.startable
      ? VideoDeliveryPhase.startable
      : VideoDeliveryPhase.preparing;
}
