part of 'progressive_device_origin.dart';

typedef ProgressiveOriginByteRange = ({int start, int end});

final class ProgressiveOriginCoverage {
  factory ProgressiveOriginCoverage.fromRequests(
    Iterable<ProgressiveOriginRequest> requests, {
    required int objectLength,
  }) {
    if (objectLength <= 0) throw ArgumentError.value(objectLength);
    final body = requests.where((request) => request.servedBytes > 0).toList();
    final served = _servedRanges(body);
    final merged = _mergedRanges(served);
    return ProgressiveOriginCoverage._(
      objectLength: objectLength,
      networkBytes: body.fold(0, (sum, item) => sum + item.servedBytes),
      uniqueBytes: _rangeBytes(merged),
      servedRanges: List.unmodifiable(served),
      missingRanges: List.unmodifiable(_missingRanges(merged, objectLength)),
      isWithinObject: served.every(
        (range) => range.start >= 0 && range.end <= objectLength,
      ),
    );
  }

  const ProgressiveOriginCoverage._({
    required this.objectLength,
    required this.networkBytes,
    required this.uniqueBytes,
    required this.servedRanges,
    required this.missingRanges,
    required this.isWithinObject,
  });

  final int objectLength;
  final int networkBytes;
  final int uniqueBytes;
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
