part of 'device_playback_probe.dart';

extension DevicePlaybackProbeRebuffer on DevicePlaybackProbe {
  double get rebufferRatio {
    final first = observations.indexWhere(_isPlaying);
    if (first < 0) return double.infinity;
    final observed = elapsed - observations[first].elapsed;
    if (observed == Duration.zero) return double.infinity;
    return _stalledDuration(first).inMicroseconds / observed.inMicroseconds;
  }

  Duration _stalledDuration(int first) {
    var result = Duration.zero;
    for (var index = first; index < observations.length; index += 1) {
      if (observations[index].observation.phase !=
          PlaybackPhase.networkStalled) {
        continue;
      }
      final end = index + 1 < observations.length
          ? observations[index + 1].elapsed
          : elapsed;
      result += end - observations[index].elapsed;
    }
    return result;
  }
}
