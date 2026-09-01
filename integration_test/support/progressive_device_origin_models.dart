part of 'progressive_device_origin.dart';

enum ProgressiveOriginValidator { none, stableStrong }

enum ProgressiveOriginAvailability { available, unavailable }

enum ProgressiveOriginRangeSemantics { coherent, ignored, malformed }

const defaultDeviceProgressiveOriginPacing =
    ProgressiveOriginPacing.perResponseDelay(Duration(milliseconds: 4));
