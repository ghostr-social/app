import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/platform/media/ffi_video_delivery_updates.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

void main() {
  test('observer error preserves readiness and the stream continues', () async {
    final native = StreamController<FfiDeliveryEvent>();
    addTearDown(native.close);
    final updates = FfiVideoDeliveryUpdates(watch: () => native.stream);
    final snapshots = <VideoDeliverySnapshot>[];
    final errors = <Object>[];
    final subscription = updates.watchDelivery().listen(
      snapshots.add,
      onError: (Object error) => errors.add(error),
    );
    addTearDown(subscription.cancel);

    native.add(_event(FfiDeliveryEventKind.progress, startable: true));
    native.add(_event(FfiDeliveryEventKind.error, detail: 'store read'));
    native.add(_event(FfiDeliveryEventKind.readiness));
    await pumpEventQueue();

    expect(snapshots.map((item) => item.phase), [
      VideoDeliveryPhase.startable,
      VideoDeliveryPhase.preparing,
    ]);
    expect(errors.single, isA<VideoDeliveryObservationException>());
  });
}

FfiDeliveryEvent _event(
  FfiDeliveryEventKind kind, {
  bool startable = false,
  String? detail,
}) => FfiDeliveryEvent(
  postId: 'media-one',
  kind: kind,
  startable: startable,
  bytesPresent: BigInt.from(64),
  totalBytes: BigInt.from(128),
  etaMs: BigInt.from(120),
  detail: detail,
);
