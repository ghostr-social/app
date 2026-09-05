import 'package:ghostr/src/rust/api/delivery_types.dart';
import 'package:ghostr/src/rust/api/focus_control.dart';

import 'live_video_log.dart';

final class LiveGatewayProbe {
  LiveGatewayProbe(this.log);
  final LiveVideoLog log;

  Future<String> resolve({required FfiFocusItem item}) async {
    final watch = Stopwatch()..start();
    log.add('gateway_resolve_started', {'deliveryId': item.postId});
    try {
      final url = await ffiPlaybackUrl(item: item);
      log.add('gateway_resolved', {
        'deliveryId': item.postId,
        'durationMs': watch.elapsedMilliseconds,
      });
      return url;
    } on Object catch (error) {
      log.add('gateway_failed', {
        'deliveryId': item.postId,
        'durationMs': watch.elapsedMilliseconds,
        'error': '$error',
      });
      rethrow;
    }
  }
}
