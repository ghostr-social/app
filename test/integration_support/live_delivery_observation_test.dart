import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';

import '../../integration_test/support/live_delivery_evidence.dart';
import '../../integration_test/support/live_video_log.dart';

void main() {
  test('repeated catalog snapshots cannot flood the live evidence stream', () {
    final log = LiveVideoLog();
    final observer = LiveDeliveryEvidence(log);
    final id = PlaybackDeliveryId.parse('active');
    final preparing = VideoDeliverySnapshot(
      deliveryId: id,
      phase: VideoDeliveryPhase.preparing,
      bytesPresent: BigInt.zero,
    );
    for (var i = 0; i < 10000; i++) {
      observer.record(preparing, currentDelivery: id.value);
    }
    observer.record(
      VideoDeliverySnapshot(
        deliveryId: id,
        phase: VideoDeliveryPhase.failed,
        bytesPresent: BigInt.zero,
        detail: 'real origin timed out',
      ),
      currentDelivery: id.value,
    );
    expect(observer.received, 10001);
    expect(observer.emitted, 2);
    expect(log.records.last['detail'], 'real origin timed out');
  });
}
