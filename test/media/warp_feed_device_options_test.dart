import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';

import '../../integration_test/support/progressive_device_origin.dart';
import '../../integration_test/support/warp_feed_device_options.dart';
import '../../integration_test/support/warp_feed_event_config.dart';

void main() {
  test('device options have one unambiguous signed-feed configuration', () {
    const options = WarpFeedDeviceOptions(
      events: SignedWarpFeedConfig(eventCount: 4),
      dataUsage: DataUsageLevel.aggressive,
      origin: WarpFeedOriginOptions(
        validator: ProgressiveOriginValidator.stableStrong,
      ),
    );

    expect(options.events.eventCount, 4);
    expect(options.dataUsage, DataUsageLevel.aggressive);
    expect(options.origin.validator, ProgressiveOriginValidator.stableStrong);
  });
}
