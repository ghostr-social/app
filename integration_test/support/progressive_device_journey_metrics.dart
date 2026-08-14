part of 'progressive_device_journey.dart';

extension ProgressiveDeviceJourneyMetrics on ProgressiveDeviceJourney {
  int get currentOriginBytes => origin.bytesServed('current');

  bool get headsRemainBlocked => origin.headsRemainBlocked;

  Set<String> get submittedPlaybackDeliveryIds =>
      _telemetry.observedDeliveryIds;

  Future<ProgressivePlaybackAdmissions> playbackAdmissions() {
    return _admissions.delta();
  }

  bool get completedRangesDoNotOverlap {
    return _doNotOverlap(origin.rangesFor('current')) &&
        _doNotOverlap(origin.rangesFor('next'));
  }
}

bool _doNotOverlap(List<({int start, int end})> ranges) {
  final ordered = [...ranges]..sort((left, right) => left.start - right.start);
  for (var index = 1; index < ordered.length; index += 1) {
    if (ordered[index - 1].end > ordered[index].start) return false;
  }
  return true;
}
