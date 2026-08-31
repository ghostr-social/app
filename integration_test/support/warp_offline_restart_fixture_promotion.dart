part of 'warp_offline_restart_fixture.dart';

typedef _OfflinePromotion = ({
  int rangedResponses,
  int uniqueBytes,
  int totalBytes,
  int duplicateBytes,
});

_OfflinePromotion _offlinePromotion(ProgressiveDeviceOrigin origin, String id) {
  final completed = origin.requestsFor(id).where((request) {
    return request.method == 'GET' &&
        request.outcome == ProgressiveOriginRequestOutcome.completed;
  }).toList();
  final coverage = ProgressiveOriginCoverage.fromRequests(
    completed,
    objectLength: origin.objectLength,
  );
  return (
    rangedResponses: completed.where((request) => request.range != null).length,
    uniqueBytes: coverage.uniqueBytes,
    totalBytes: coverage.objectLength,
    duplicateBytes: coverage.duplicateBytes,
  );
}

bool _isCompletePromotion(_OfflinePromotion value) {
  return value.rangedResponses > 1 &&
      value.uniqueBytes == value.totalBytes &&
      value.duplicateBytes == 0;
}
