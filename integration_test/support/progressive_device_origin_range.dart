part of 'progressive_device_origin.dart';

({int start, int end})? _requestedRange(HttpRequest request, int total) {
  final raw = request.headers.value(HttpHeaders.rangeHeader);
  if (raw == null || !raw.startsWith('bytes=')) return null;
  final bounds = raw.substring(6).split('-');
  final start = int.tryParse(bounds.first) ?? 0;
  final inclusiveEnd = int.tryParse(bounds.last) ?? total - 1;
  return (
    start: start.clamp(0, total - 1),
    end: (inclusiveEnd + 1).clamp(1, total),
  );
}

void _setContentRange(
  HttpResponse response,
  ({int start, int end}) range,
  int total,
) {
  response.headers.set(
    HttpHeaders.contentRangeHeader,
    'bytes ${range.start}-${range.end - 1}/$total',
  );
}
