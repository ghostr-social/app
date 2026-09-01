import 'package:ghostr/features/settings/domain/data_usage_level.dart';

import 'progressive_device_origin.dart';
import 'warp_feed_event_config.dart';

final class WarpFeedOriginOptions {
  const WarpFeedOriginOptions({
    this.pacing = defaultDeviceProgressiveOriginPacing,
    this.validator = ProgressiveOriginValidator.none,
    this.rangeSemanticsById = const {},
  });

  final ProgressiveOriginPacing pacing;
  final ProgressiveOriginValidator validator;
  final Map<String, ProgressiveOriginRangeSemantics> rangeSemanticsById;
}

final class WarpFeedDeviceOptions {
  const WarpFeedDeviceOptions({
    this.events = const SignedWarpFeedConfig(),
    this.dataUsage = DataUsageLevel.balanced,
    this.origin = const WarpFeedOriginOptions(),
  });

  final SignedWarpFeedConfig events;
  final DataUsageLevel dataUsage;
  final WarpFeedOriginOptions origin;
}
