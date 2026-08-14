import 'dart:async';
import 'dart:developer';
import 'dart:io';
import 'dart:typed_data';

import 'progressive_mp4_fixture.dart';

part 'progressive_device_origin_response.dart';
part 'progressive_device_origin_request.dart';

final class ProgressiveDeviceOrigin {
  ProgressiveDeviceOrigin._(
    this._server,
    this._responseChunkBytes,
    this._responseChunkDelay,
  );

  static Future<ProgressiveDeviceOrigin> start({
    int responseChunkBytes = 16 * 1024,
    Duration responseChunkDelay = Duration.zero,
  }) async {
    if (responseChunkBytes <= 0) throw ArgumentError.value(responseChunkBytes);
    final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    final origin = ProgressiveDeviceOrigin._(
      server,
      responseChunkBytes,
      responseChunkDelay,
    );
    origin._subscription = server.listen(origin._dispatch);
    return origin;
  }

  final HttpServer _server;
  final int _responseChunkBytes;
  final Duration _responseChunkDelay;
  final requests = <ProgressiveOriginRequest>[];
  final _completed = <ProgressiveOriginRequest>[];
  final _heldHeads = <HttpResponse>[];
  final _servedBytes = <String, int>{};
  late final StreamSubscription<HttpRequest> _subscription;

  Uri get origin =>
      Uri(scheme: 'http', host: _server.address.address, port: _server.port);

  Uri urlFor(String id) =>
      Uri.parse('http://${_server.address.address}:${_server.port}/$id.mp4');

  int bytesServed(String id) => _servedBytes['/$id.mp4'] ?? 0;

  List<({int start, int end})> rangesFor(String id) => _completed
      .where((request) => request.path == '/$id.mp4')
      .map((request) => request.range)
      .whereType<({int start, int end})>()
      .toList();

  bool get headsRemainBlocked => _heldHeads.isNotEmpty;

  void _dispatch(HttpRequest request) => unawaited(_serve(request));

  Future<void> _serve(HttpRequest request) async {
    try {
      await _handle(request);
    } on Object catch (error, stackTrace) {
      log(
        'Progressive device origin request failed.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  Future<void> _handle(HttpRequest request) async {
    final range = _requestedRange(request, ProgressiveMp4Fixture.bytes.length);
    final entry = ProgressiveOriginRequest(
      request.method,
      request.uri.path,
      range,
    );
    requests.add(entry);
    if (request.method == 'HEAD') {
      _heldHeads.add(request.response);
      return;
    }
    final completed = await _write(request.response, range, entry.path);
    if (completed) _completed.add(entry);
  }

  void _recordBytes(String path, int count) {
    _servedBytes.update(path, (total) => total + count, ifAbsent: () => count);
  }

  Future<void> close() async {
    for (final response in _heldHeads) {
      await response.close();
    }
    await _subscription.cancel();
    await _server.close(force: true);
  }
}
