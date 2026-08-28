part of 'progressive_device_origin.dart';

typedef _ProgressiveOriginLinkProfileData = ({
  int generation,
  int bandwidthKbps,
  Duration appliedAt,
  int appliedAtEpochMs,
  Set<int> activeRequestSequences,
  Set<String> activePaths,
  int bytesSentAtApplication,
});

final class ProgressiveOriginLinkProfile {
  ProgressiveOriginLinkProfile._(_ProgressiveOriginLinkProfileData data)
    : generation = data.generation,
      bandwidthKbps = data.bandwidthKbps,
      appliedAt = data.appliedAt,
      appliedAtEpochMs = data.appliedAtEpochMs,
      activeRequestSequences = data.activeRequestSequences,
      activePaths = data.activePaths,
      bytesSentAtApplication = data.bytesSentAtApplication;

  final int generation;
  final int bandwidthKbps;
  final Duration appliedAt;
  final int appliedAtEpochMs;
  final Set<int> activeRequestSequences;
  final Set<String> activePaths;
  final int bytesSentAtApplication;
}
