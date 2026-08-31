part of 'warp_feed_playback_journey.dart';

typedef WarpProgressivePromotionEvidence = ({
  int rangedResponses,
  int uniqueBytes,
  int totalBytes,
  int duplicateBytes,
});

extension WarpFeedPlaybackJourneyPromotionWait on WarpFeedPlaybackJourney {
  Future<WarpProgressivePromotionEvidence> waitForProgressivePromotion(
    WidgetTester tester,
    String id,
  ) async {
    var evidence = _promotionEvidence(resources.origin, id);
    await _wait(tester, () {
      evidence = _promotionEvidence(resources.origin, id);
      return _isCompletePromotion(evidence);
    });
    await waitForNativeStoreCoverage(tester, [id]);
    return evidence;
  }
}

WarpProgressivePromotionEvidence _promotionEvidence(
  ProgressiveDeviceOrigin origin,
  String id,
) {
  final completed = origin.requestsFor(id).where(_isCompletedGet).toList();
  final coverage = ProgressiveOriginCoverage.fromRequests(
    completed,
    objectLength: origin.objectLength,
  );
  return (
    rangedResponses: completed
        .where((request) => _isSparseRange(request, origin.objectLength))
        .length,
    uniqueBytes: coverage.uniqueBytes,
    totalBytes: coverage.objectLength,
    duplicateBytes: coverage.duplicateBytes,
  );
}

bool _isCompletePromotion(WarpProgressivePromotionEvidence evidence) {
  return evidence.rangedResponses > 1 &&
      evidence.uniqueBytes == evidence.totalBytes &&
      evidence.duplicateBytes == 0;
}

bool _isCompletedGet(ProgressiveOriginRequest request) {
  return request.method == 'GET' &&
      request.outcome == ProgressiveOriginRequestOutcome.completed;
}

bool _isSparseRange(ProgressiveOriginRequest request, int objectLength) {
  return request.range != null &&
      request.servedBytes > 0 &&
      request.servedBytes < objectLength;
}
