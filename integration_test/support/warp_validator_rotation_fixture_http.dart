part of 'warp_validator_rotation_fixture.dart';

_RotationResponsePlan _responsePlan(
  WarpValidatorRequest request,
  int length,
  String validator,
) {
  final range = _parseRotationRange(request.range, length);
  final stale = request.ifRange != null && request.ifRange != validator;
  if (range == null || stale) {
    return (status: HttpStatus.ok, start: 0, end: length, contentRange: null);
  }
  return (
    status: HttpStatus.partialContent,
    start: range.start,
    end: range.end,
    contentRange: 'bytes ${range.start}-${range.end - 1}/$length',
  );
}

({int start, int end})? _parseRotationRange(String? value, int length) {
  if (value == null) return null;
  final match = RegExp(r'^bytes=(\d+)-(\d*)$').firstMatch(value);
  if (match == null) return null;
  final start = int.parse(match.group(1)!);
  final rawEnd = match.group(2)!;
  final end = rawEnd.isEmpty ? length : int.parse(rawEnd) + 1;
  if (start >= length || end <= start) return null;
  return (start: start, end: min(end, length));
}

void _configureRotationResponse(
  HttpResponse response,
  _RotationResponsePlan plan,
  String validator,
) {
  response.statusCode = plan.status;
  response.bufferOutput = false;
  response.headers.contentType = ContentType('video', 'mp4');
  response.headers.contentLength = plan.end - plan.start;
  response.headers.set(HttpHeaders.acceptRangesHeader, 'bytes');
  response.headers.set(HttpHeaders.etagHeader, validator);
  if (plan.contentRange case final value?) {
    response.headers.set(HttpHeaders.contentRangeHeader, value);
  }
}
