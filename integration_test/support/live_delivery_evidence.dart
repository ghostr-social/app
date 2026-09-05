import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';

import 'live_video_log.dart';

final class LiveDeliveryEvidence {
  LiveDeliveryEvidence(this.log);
  final LiveVideoLog log;
  final _latest = <String, String>{};
  int received = 0;
  int emitted = 0;

  void record(VideoDeliverySnapshot event, {String? currentDelivery}) {
    received++;
    final id = event.deliveryId.value;
    final fingerprint =
        '${event.phase}:${event.bytesPresent}:'
        '${event.totalBytes}:${event.detail}';
    if (_latest[id] == fingerprint) return;
    _latest[id] = fingerprint;
    if (_latest.length > 2048) _latest.remove(_latest.keys.first);
    if (id != currentDelivery && event.phase == VideoDeliveryPhase.preparing) {
      return;
    }
    emitted++;
    log.add('delivery', {
      'deliveryId': id,
      'phase': event.phase.name,
      'bytes': event.bytesPresent.toString(),
      'total': event.totalBytes?.toString(),
      'etaMs': event.eta?.inMilliseconds,
      'detail': event.detail,
    });
  }

  void summarize() =>
      log.add('delivery_stream', {'received': received, 'emitted': emitted});
}
