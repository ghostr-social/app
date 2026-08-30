part of 'progressive_device_origin.dart';

typedef ProgressiveOriginByteRange = ({int start, int end});

final class ProgressiveOriginCoverage {
  factory ProgressiveOriginCoverage.fromRequests(
    Iterable<ProgressiveOriginRequest> requests, {
    required int objectLength,
  }) {
    if (objectLength <= 0) throw ArgumentError.value(objectLength);
    final data = _coverageData(requests, objectLength);
    return ProgressiveOriginCoverage._(objectLength: objectLength, data: data);
  }

  ProgressiveOriginCoverage._({
    required this.objectLength,
    required _ProgressiveOriginCoverageData data,
  }) : networkBytes = data.networkBytes,
       uniqueBytes = data.uniqueBytes,
       completedDuplicateBytes = data.completedDuplicateBytes,
       cancellationAttributedDuplicateBytes =
           data.cancellationAttributedDuplicateBytes,
       servedRanges = data.servedRanges,
       missingRanges = data.missingRanges,
       isWithinObject = data.isWithinObject;

  final int objectLength;
  final int networkBytes;
  final int uniqueBytes;
  final int completedDuplicateBytes;
  final int cancellationAttributedDuplicateBytes;
  final List<ProgressiveOriginByteRange> servedRanges;
  final List<ProgressiveOriginByteRange> missingRanges;
  final bool isWithinObject;

  int get duplicateBytes => networkBytes - uniqueBytes;
  bool get isComplete => isWithinObject && missingRanges.isEmpty;
  bool get isExact =>
      isComplete &&
      duplicateBytes == 0 &&
      uniqueBytes == objectLength &&
      networkBytes == objectLength;

  bool isReplayCompleteWithin({required int cancellationOverlapBudgetBytes}) {
    return hasReplayIntegrityWithin(
          cancellationOverlapBudgetBytes: cancellationOverlapBudgetBytes,
        ) &&
        isComplete &&
        uniqueBytes == objectLength;
  }

  bool hasReplayIntegrityWithin({required int cancellationOverlapBudgetBytes}) {
    _requireNonNegative(cancellationOverlapBudgetBytes);
    return isWithinObject &&
        completedDuplicateBytes == 0 &&
        duplicateBytes == cancellationAttributedDuplicateBytes &&
        cancellationAttributedDuplicateBytes <= cancellationOverlapBudgetBytes;
  }
}

typedef _ProgressiveOriginCoverageData = ({
  int networkBytes,
  int uniqueBytes,
  int completedDuplicateBytes,
  int cancellationAttributedDuplicateBytes,
  List<ProgressiveOriginByteRange> servedRanges,
  List<ProgressiveOriginByteRange> missingRanges,
  bool isWithinObject,
});

_ProgressiveOriginCoverageData _coverageData(
  Iterable<ProgressiveOriginRequest> requests,
  int objectLength,
) {
  final body = requests.where((request) => request.servedBytes > 0).toList();
  final served = _servedRanges(body);
  final merged = _mergedRanges(served);
  final completed = _completedRanges(body);
  final withoutCancellation = _uncancelledRanges(body);
  final duplicateBytes = _duplicateBytes(served);
  return (
    networkBytes: body.fold(0, (sum, item) => sum + item.servedBytes),
    uniqueBytes: _rangeBytes(merged),
    completedDuplicateBytes: _duplicateBytes(completed),
    cancellationAttributedDuplicateBytes:
        duplicateBytes - _duplicateBytes(withoutCancellation),
    servedRanges: List.unmodifiable(served),
    missingRanges: List.unmodifiable(_missingRanges(merged, objectLength)),
    isWithinObject: served.every(
      (range) => range.start >= 0 && range.end <= objectLength,
    ),
  );
}

List<ProgressiveOriginByteRange> _completedRanges(
  Iterable<ProgressiveOriginRequest> requests,
) {
  return _servedRanges(
    requests.where(
      (request) => request.outcome == ProgressiveOriginRequestOutcome.completed,
    ),
  );
}

List<ProgressiveOriginByteRange> _uncancelledRanges(
  Iterable<ProgressiveOriginRequest> requests,
) {
  return _servedRanges(
    requests.where(
      (request) =>
          request.outcome != ProgressiveOriginRequestOutcome.clientCanceled,
    ),
  );
}

List<ProgressiveOriginByteRange> _servedRanges(
  Iterable<ProgressiveOriginRequest> requests,
) {
  return requests
      .map((request) {
        final start = request.range?.start ?? 0;
        return (start: start, end: start + request.servedBytes);
      })
      .toList(growable: false);
}

List<ProgressiveOriginByteRange> _mergedRanges(
  List<ProgressiveOriginByteRange> ranges,
) {
  final ordered = [...ranges]..sort((left, right) => left.start - right.start);
  final merged = <ProgressiveOriginByteRange>[];
  for (final range in ordered) {
    if (merged.isEmpty || merged.last.end < range.start) {
      merged.add(range);
      continue;
    }
    final previous = merged.removeLast();
    merged.add((start: previous.start, end: max(previous.end, range.end)));
  }
  return merged;
}

List<ProgressiveOriginByteRange> _missingRanges(
  List<ProgressiveOriginByteRange> ranges,
  int length,
) {
  final missing = <ProgressiveOriginByteRange>[];
  var cursor = 0;
  for (final range in ranges) {
    final start = range.start.clamp(0, length);
    final end = range.end.clamp(0, length);
    if (start > cursor) missing.add((start: cursor, end: start));
    if (end > cursor) cursor = end;
  }
  if (cursor < length) missing.add((start: cursor, end: length));
  return missing;
}

int _rangeBytes(Iterable<ProgressiveOriginByteRange> ranges) {
  return ranges.fold(0, (sum, range) => sum + range.end - range.start);
}

int _duplicateBytes(List<ProgressiveOriginByteRange> ranges) {
  return _rangeBytes(ranges) - _rangeBytes(_mergedRanges(ranges));
}

void _requireNonNegative(int value) {
  if (value < 0) throw ArgumentError.value(value);
}

bool progressiveReplayCancellationOverlapWithin(
  Iterable<ProgressiveOriginCoverage> coverages, {
  required int budgetBytes,
}) {
  _requireNonNegative(budgetBytes);
  var total = 0;
  for (final coverage in coverages) {
    total += coverage.cancellationAttributedDuplicateBytes;
    if (total > budgetBytes) return false;
  }
  return true;
}
