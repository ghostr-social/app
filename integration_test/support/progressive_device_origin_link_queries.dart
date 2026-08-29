part of 'progressive_device_origin.dart';

extension ProgressiveOriginLinkControl on ProgressiveDeviceOrigin {
  ProgressiveOriginLinkProfile? get currentLinkProfile => _pacing.current;

  Set<int> get activeIncompleteRequestSequences =>
      Set.unmodifiable(_activeIncompleteRequests().keys);

  int? requestSequenceFor(ProgressiveOriginRequest request) {
    return _requestSequences[request];
  }

  List<ProgressiveOriginChunkEvent> confirmedChunkEventsFor(String id) {
    return List.unmodifiable(
      _pacing.events.where(
        (event) => event.path == '/$id.mp4' && event.confirmedAtEpochMs != null,
      ),
    );
  }

  ProgressiveOriginLinkProfile setBandwidthKbps(int bandwidthKbps) {
    return _pacing.change(bandwidthKbps, _activeIncompleteRequests());
  }

  ProgressiveOriginLinkWindow linkWindow(int generation) {
    final events = _pacing.events
        .where(
          (item) =>
              item.profileGeneration == generation &&
              item.confirmedAtEpochMs != null,
        )
        .toList(growable: false);
    if (events.isEmpty) return _emptyLinkWindow(generation);
    final duration = events.last.sentAt - events.first.serviceStartedAt;
    final bytes = events.fold(0, (sum, item) => sum + item.bytes);
    return ProgressiveOriginLinkWindow._((
      generation: generation,
      bytes: bytes,
      duration: duration,
      achievedBandwidthKbps: _achievedBandwidth(bytes, duration),
      confirmedAtEpochMs: progressiveConfirmedWindowFence(
        events.map((event) => event.confirmedAtEpochMs!),
      ),
      events: List.unmodifiable(events),
    ));
  }

  bool requestSpansProfiles(int request, Set<int> generations) {
    final observed = _pacing.events
        .where(
          (item) =>
              item.requestSequence == request &&
              item.confirmedAtEpochMs != null,
        )
        .map((item) => item.profileGeneration)
        .toSet();
    return observed.containsAll(generations);
  }

  Map<int, ProgressiveOriginRequest> _activeIncompleteRequests() {
    return Map.unmodifiable({
      for (final entry in _requestSequences.entries)
        if (_isActiveIncomplete(entry.key)) entry.value: entry.key,
    });
  }

  bool _isActiveIncomplete(ProgressiveOriginRequest request) {
    final expected = request.range == null
        ? objectLength
        : request.range!.end - request.range!.start;
    return request.method != 'HEAD' &&
        request.outcome == ProgressiveOriginRequestOutcome.serving &&
        request.servedBytes < expected;
  }
}

int progressiveConfirmedWindowFence(Iterable<int> epochs) {
  final iterator = epochs.iterator;
  if (!iterator.moveNext()) {
    throw StateError('A confirmation fence needs data.');
  }
  var latest = iterator.current;
  while (iterator.moveNext()) {
    if (iterator.current > latest) latest = iterator.current;
  }
  return latest;
}

ProgressiveOriginLinkWindow _emptyLinkWindow(int generation) {
  return ProgressiveOriginLinkWindow._((
    generation: generation,
    bytes: 0,
    duration: Duration.zero,
    achievedBandwidthKbps: 0,
    confirmedAtEpochMs: 0,
    events: const [],
  ));
}

int _achievedBandwidth(int bytes, Duration duration) {
  if (duration <= Duration.zero) return 0;
  return bytes * 8000 ~/ duration.inMicroseconds.clamp(1, 1 << 62);
}
