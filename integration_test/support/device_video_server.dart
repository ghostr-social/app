import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'deterministic_hls_fixture.dart';
import 'device_video_scenario.dart';

part 'device_video_server_impairments.dart';
part 'device_video_server_responses.dart';

final class DeviceVideoServer {
  DeviceVideoServer._(this._server, this.scenario);

  static Future<DeviceVideoServer> start(DeviceVideoScenario scenario) async {
    final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    final result = DeviceVideoServer._(server, scenario);
    server.listen(result._handleSafely);
    return result;
  }

  final HttpServer _server;
  final DeviceVideoScenario scenario;
  final _requests = <String, int>{};
  final _sessionNames = <String, String>{};
  final _requestedSessions = <String>{};
  final _heldResponseReleased = Completer<void>();

  int impairedResponses = 0;
  int disconnects = 0;
  int manifestFailures = 0;
  int successfulManifestResponses = 0;
  int heldResponses = 0;
  int cancellationWasteBytes = 0;
  bool isResponseHeld = false;

  Set<String> get requestedSessions =>
      Set<String>.unmodifiable(_requestedSessions);

  Uri playbackUri(String videoId) {
    final session = _sessionId(videoId);
    _sessionNames[session] = videoId;
    return Uri.parse(
      'http://${_server.address.address}:${_server.port}/hls/$session/index.m3u8',
    );
  }

  int requestsFor(String asset) => _requests[asset] ?? 0;

  void releaseHeldResponse() {
    if (!_heldResponseReleased.isCompleted) _heldResponseReleased.complete();
    isResponseHeld = false;
  }

  Future<void> close() async {
    releaseHeldResponse();
    await _server.close(force: true);
  }

  Future<void> _handleSafely(HttpRequest request) async {
    try {
      await _handle(request);
    } on Object {
      final attemptedBytes = request.response.contentLength;
      if (attemptedBytes > 0) cancellationWasteBytes += attemptedBytes;
    }
  }

  Future<void> _handle(HttpRequest request) async {
    final parts = request.uri.pathSegments;
    if (parts.length != 3 || parts.first != 'hls') {
      return _notFound(request.response);
    }
    final asset = parts.last;
    _requestedSessions.add(_sessionNames[parts[1]] ?? parts[1]);
    _requests.update(asset, (value) => value + 1, ifAbsent: () => 1);
    await _applyDelay(asset);
    if (_failFirstManifest(asset)) return _serviceUnavailable(request.response);
    await _waitForHeldResponse(asset);
    final bytes = _asset(asset);
    if (bytes == null) return _notFound(request.response);
    if (_disconnect(asset)) return _abort(request.response, bytes);
    await _write(request.response, asset, bytes);
  }
}
