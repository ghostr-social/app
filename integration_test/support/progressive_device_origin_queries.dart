part of 'progressive_device_origin.dart';

extension ProgressiveDeviceOriginQueries on ProgressiveDeviceOrigin {
  bool get isUnavailable =>
      _availability == ProgressiveOriginAvailability.unavailable;

  Uri get origin =>
      Uri(scheme: 'http', host: _server.address.address, port: _server.port);

  Uri urlFor(String id) =>
      Uri.parse('http://${_server.address.address}:${_server.port}/$id.mp4');

  int bytesServed(String id) => _servedBytes['/$id.mp4'] ?? 0;

  int get objectLength => ProgressiveMp4Fixture.bytes.length;

  List<ProgressiveOriginRequest> requestsFor(String id) {
    return requests
        .where((request) => request.path == '/$id.mp4')
        .toList(growable: false);
  }

  ProgressiveOriginCoverage coverageFor(String id, {int? requestCount}) {
    final matching = requestsFor(id);
    final selected = requestCount == null
        ? matching
        : matching.take(requestCount);
    return ProgressiveOriginCoverage.fromRequests(
      selected,
      objectLength: objectLength,
    );
  }

  List<({int start, int end})> rangesFor(String id) => _completed
      .where((request) => request.path == '/$id.mp4')
      .map((request) => request.range)
      .whereType<({int start, int end})>()
      .toList();
}
