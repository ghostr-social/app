import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/platform/media/ffi_video_delivery_updates.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

void main() {
  test('opens the native delivery stream only when listened to', () async {
    final native = StreamController<FfiDeliveryEvent>();
    addTearDown(native.close);
    var watchCalls = 0;
    final updates = FfiVideoDeliveryUpdates(
      watch: () {
        watchCalls += 1;
        return native.stream;
      },
    );

    expect(watchCalls, 0);
    final delivery = updates.watchDelivery();
    expect(watchCalls, 0);

    final subscription = delivery.listen((_) {});
    addTearDown(subscription.cancel);
    await Future<void>.delayed(Duration.zero);
    expect(watchCalls, 1);
  });

  test(
    'maps native progress and errors into typed delivery snapshots',
    () async {
      final native = StreamController<FfiDeliveryEvent>();
      addTearDown(native.close);
      final updates = FfiVideoDeliveryUpdates(watch: () => native.stream);
      final received = updates.watchDelivery().take(2).toList();

      native.add(event(startable: true, kind: FfiDeliveryEventKind.progress));
      native.add(event(startable: false, kind: FfiDeliveryEventKind.error));

      final snapshots = await received;
      expect(snapshots.first.deliveryId, PlaybackDeliveryId.parse('media-one'));
      expect(snapshots.first.phase, VideoDeliveryPhase.startable);
      expect(snapshots.first.eta, const Duration(milliseconds: 120));
      expect(snapshots.last.phase, VideoDeliveryPhase.failed);
      expect(snapshots.last.detail, 'store failure');
    },
  );
}

FfiDeliveryEvent event({
  required bool startable,
  required FfiDeliveryEventKind kind,
}) {
  return FfiDeliveryEvent(
    postId: 'media-one',
    kind: kind,
    startable: startable,
    bytesPresent: BigInt.from(64),
    totalBytes: BigInt.from(128),
    etaMs: BigInt.from(120),
    detail: kind == FfiDeliveryEventKind.error ? 'store failure' : null,
  );
}
