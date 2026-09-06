import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/platform/media/ffi_video_delivery_updates.dart';

import 'live_delivery_evidence.dart';
import 'live_focus_probe.dart';

// Observe the production subscription without adding a second Rust watcher.
final class LiveDeliveryUpdates implements VideoDeliveryUpdates {
  LiveDeliveryUpdates(this.evidence, this.focus);

  final LiveDeliveryEvidence evidence;
  final LiveFocusProbe focus;
  final _native = FfiVideoDeliveryUpdates();

  @override
  Stream<VideoDeliverySnapshot> watchDelivery() =>
      _native.watchDelivery().map((event) {
        final active = focus.current?.current.id.value;
        evidence.record(
          event,
          currentDelivery: active == null
              ? null
              : focus.probe.deliveryForEvent(active)?.value,
        );
        return event;
      });
}
