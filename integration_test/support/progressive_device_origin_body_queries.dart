part of 'progressive_device_origin.dart';

extension ProgressiveDeviceOriginBodyQueries on ProgressiveDeviceOrigin {
  List<String> get bodyRequestedIds {
    final ids = <String>{};
    for (final request in requests) {
      final id = _bodyRequestId(request);
      if (id != null) ids.add(id);
    }
    return List.unmodifiable(ids);
  }
}

String? _bodyRequestId(ProgressiveOriginRequest request) {
  if (request.method != 'GET' || !request.path.endsWith('.mp4')) return null;
  final name = Uri(path: request.path).pathSegments.last;
  return name.substring(0, name.length - '.mp4'.length);
}
