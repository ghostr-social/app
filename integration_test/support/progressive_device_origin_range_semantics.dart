part of 'progressive_device_origin.dart';

typedef _ProgressiveOriginResponsePlan = ({
  int statusCode,
  ({int start, int end}) span,
  String? contentRange,
});

extension _ProgressiveDeviceOriginRangeSemantics on ProgressiveDeviceOrigin {
  _ProgressiveOriginResponsePlan _responsePlan(
    String path,
    ({int start, int end})? requested,
    int total,
  ) {
    if (requested == null) return _fullResponse(total);
    return switch (_rangeSemanticsFor(path)) {
      ProgressiveOriginRangeSemantics.coherent => (
        statusCode: HttpStatus.partialContent,
        span: requested,
        contentRange: _contentRangeValue(requested, total),
      ),
      ProgressiveOriginRangeSemantics.ignored => _fullResponse(total),
      ProgressiveOriginRangeSemantics.malformed => (
        statusCode: HttpStatus.partialContent,
        span: requested,
        contentRange: 'invalid',
      ),
    };
  }

  ProgressiveOriginRangeSemantics _rangeSemanticsFor(String path) {
    final filename = Uri(path: path).pathSegments.last;
    final id = filename.substring(0, filename.length - '.mp4'.length);
    return _rangeSemanticsById[id] ?? _rangeSemantics;
  }
}

_ProgressiveOriginResponsePlan _fullResponse(int total) => (
  statusCode: HttpStatus.ok,
  span: (start: 0, end: total),
  contentRange: null,
);
