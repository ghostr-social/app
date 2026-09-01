import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';
import '../../integration_test/support/warp_feed_event_config.dart';
import '../../integration_test/support/warp_feed_events.dart';

void main() {
  test(
    'signed next event emits its primary before the rescue fallback',
    () async {
      final origin = await ProgressiveDeviceOrigin.start();
      addTearDown(origin.close);
      final events = await signedWarpFeedEvents(
        origin,
        config: const SignedWarpFeedConfig(
          eventCount: 3,
          candidateLayout: WarpFeedCandidateLayout.nextWithRescue,
        ),
      );

      final imeta = events[1].tags.singleWhere((tag) => tag.first == 'imeta');

      expect(imeta[1], 'url ${origin.urlFor('next')}');
      expect(imeta[2], 'fallback ${origin.urlFor('next-rescue')}');
    },
  );
}
