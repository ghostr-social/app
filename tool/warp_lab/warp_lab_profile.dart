import 'package:ghostr/features/settings/domain/data_usage_level.dart';

import '../../integration_test/support/progressive_device_origin.dart';
import 'warp_lab_destination.dart';

final class WarpLabProfile {
  const WarpLabProfile({
    required this.eventCount,
    required this.dataUsage,
    required this.responseChunkDelay,
    required this.validator,
  });

  factory WarpLabProfile.forDestination(WarpLabDestination destination) {
    return switch (destination) {
      WarpLabDestination.feedPlayback => _feed,
      WarpLabDestination.rapidSwipes => _rapid,
      WarpLabDestination.networkEvidence => _network,
      WarpLabDestination.menu => throw ArgumentError.value(destination),
    };
  }

  static const _feed = WarpLabProfile(
    eventCount: 3,
    dataUsage: DataUsageLevel.balanced,
    responseChunkDelay: Duration(milliseconds: 4),
    validator: ProgressiveOriginValidator.none,
  );
  static const _rapid = WarpLabProfile(
    eventCount: 7,
    dataUsage: DataUsageLevel.aggressive,
    responseChunkDelay: Duration(milliseconds: 100),
    validator: ProgressiveOriginValidator.stableStrong,
  );
  static const _network = WarpLabProfile(
    eventCount: 3,
    dataUsage: DataUsageLevel.balanced,
    responseChunkDelay: Duration(milliseconds: 4),
    validator: ProgressiveOriginValidator.none,
  );

  final int eventCount;
  final DataUsageLevel dataUsage;
  final Duration responseChunkDelay;
  final ProgressiveOriginValidator validator;
}
