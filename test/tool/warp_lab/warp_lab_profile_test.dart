import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';

import '../../../integration_test/support/progressive_device_origin.dart';
import '../../../tool/warp_lab/warp_lab_destination.dart';
import '../../../tool/warp_lab/warp_lab_profile.dart';

void main() {
  test('maps every route to its matching Android journey conditions', () {
    final feed = WarpLabProfile.forDestination(WarpLabDestination.feedPlayback);
    final rapid = WarpLabProfile.forDestination(WarpLabDestination.rapidSwipes);
    final network = WarpLabProfile.forDestination(
      WarpLabDestination.networkEvidence,
    );

    expect(feed.eventCount, 3);
    expect(feed.dataUsage, DataUsageLevel.balanced);
    expect(feed.validator, ProgressiveOriginValidator.none);
    expect(feed.responseChunkDelay, const Duration(milliseconds: 4));
    expect(rapid.eventCount, 7);
    expect(rapid.dataUsage, DataUsageLevel.aggressive);
    expect(rapid.validator, ProgressiveOriginValidator.stableStrong);
    expect(rapid.responseChunkDelay, const Duration(milliseconds: 100));
    expect(network.eventCount, 3);
    expect(network.dataUsage, DataUsageLevel.balanced);
    expect(network.validator, ProgressiveOriginValidator.none);
    expect(network.responseChunkDelay, const Duration(milliseconds: 4));
  });
}
