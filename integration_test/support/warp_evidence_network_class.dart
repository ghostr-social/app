part of 'warp_evidence_models.dart';

enum WarpNetworkClass { unavailable, wifi, cellular, wired, constrained }

WarpNetworkClass _warpNetworkClass(String value) => switch (value) {
  'Unavailable' => WarpNetworkClass.unavailable,
  'Wifi' => WarpNetworkClass.wifi,
  'Cellular' => WarpNetworkClass.cellular,
  'Wired' => WarpNetworkClass.wired,
  'Constrained' => WarpNetworkClass.constrained,
  _ => throw FormatException('Unknown network class: $value'),
};
