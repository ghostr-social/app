import 'package:ghostr/core/media/playback_delivery_id.dart';
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
    yield* _watch();
  }
}

VideoDeliverySnapshot _snapshot(FfiDeliveryEvent event) {
  return VideoDeliverySnapshot(
    deliveryId: PlaybackDeliveryId.parse(event.postId),
    phase: _phase(event),
    bytesPresent: event.bytesPresent,
    totalBytes: event.totalBytes,
    eta: event.etaMs == null
        ? null
        : Duration(milliseconds: event.etaMs!.toInt()),
    detail: event.detail,
  );
}

VideoDeliveryPhase _phase(FfiDeliveryEvent event) {
  if (event.kind == FfiDeliveryEventKind.error) {
    return VideoDeliveryPhase.failed;
  }
  return event.startable
      ? VideoDeliveryPhase.startable
      : VideoDeliveryPhase.preparing;
}
