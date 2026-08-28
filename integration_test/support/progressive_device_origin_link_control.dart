part of 'progressive_device_origin.dart';

extension _ProgressiveOriginPacerControl on _ProgressiveOriginPacer {
  ProgressiveOriginLinkProfile change(
    int bandwidthKbps,
    Map<int, ProgressiveOriginRequest> active,
  ) {
    if (_bandwidthKbps == null) throw StateError('The origin is not shared.');
    if (bandwidthKbps <= 0) throw ArgumentError.value(bandwidthKbps);
    _bandwidthKbps = bandwidthKbps;
    return _current = _profile(bandwidthKbps, active);
  }

  ProgressiveOriginLinkProfile _profile(
    int bandwidthKbps,
    Map<int, ProgressiveOriginRequest> active,
  ) => ProgressiveOriginLinkProfile._((
    generation: ++_generation,
    bandwidthKbps: bandwidthKbps,
    appliedAt: _clock.elapsed,
    appliedAtEpochMs: DateTime.now().millisecondsSinceEpoch,
    activeRequestSequences: Set.unmodifiable(active.keys),
    activePaths: Set.unmodifiable(active.values.map((item) => item.path)),
    bytesSentAtApplication: active.values.fold(
      0,
      (sum, item) => sum + item.servedBytes,
    ),
  ));
}
