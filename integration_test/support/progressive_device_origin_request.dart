part of 'progressive_device_origin.dart';

final class ProgressiveOriginRequest {
  const ProgressiveOriginRequest(this.method, this.path, this.range);

  final String method;
  final String path;
  final ({int start, int end})? range;
}
