import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/platform/media/ffi_video_delivery_updates.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

void main() {
  test('maps only a complete exact HLS authority', () async {
    final events = StreamController<FfiDeliveryEvent>();
    addTearDown(events.close);
    final updates = FfiVideoDeliveryUpdates(watch: () => events.stream);
    final received = updates.watchDelivery().first;

    events.add(_event());
    final snapshot = await received;

    expect(snapshot.authority, isNull);
    expect(
      snapshot.hlsAuthority?.deliveryId,
      PlaybackDeliveryId.parse('stream'),
    );
    expect(
      snapshot.hlsAuthority?.representationId,
      VideoRepresentationId.parse('a' * 64),
    );
    expect(snapshot.hlsAuthority?.assetRevision.value, BigInt.from(7));
  });
}

FfiDeliveryEvent _event() => FfiDeliveryEvent(
  postId: 'stream',
  kind: FfiDeliveryEventKind.readiness,
  startable: true,
  bytesPresent: BigInt.from(128),
  hlsDeliveryId: 'stream',
  hlsRepresentationId: 'a' * 64,
  hlsAssetRevision: BigInt.from(7),
);
