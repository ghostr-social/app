import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:math' show min;
import 'dart:typed_data';

import 'progressive_mp4_fixture.dart';

part 'warp_validator_rotation_fixture_bytes.dart';
part 'warp_validator_rotation_fixture_http.dart';
part 'warp_validator_rotation_fixture_hold.dart';
part 'warp_validator_rotation_fixture_models.dart';
part 'warp_validator_rotation_fixture_response.dart';

final class WarpValidatorRotationFixture {
  WarpValidatorRotationFixture._(this._server, this._holdFirstGeneration);

  static Future<WarpValidatorRotationFixture> start({
    bool holdFirstGeneration = true,
  }) async {
    final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    final fixture = WarpValidatorRotationFixture._(server, holdFirstGeneration);
    fixture._subscription = server.listen(fixture._dispatch);
    return fixture;
  }

  static const _firstValidator = '"warp-rotation-v1"';
  static const _secondValidator = '"warp-rotation-v2"';
  static final _firstBytes = _rotationFirstBytes();
  static final _secondBytes = _rotationSecondBytes(_firstBytes);

  final HttpServer _server;
  final bool _holdFirstGeneration;
  final _releaseFirst = Completer<void>();
  final requests = <WarpValidatorRequest>[];
  final redirectTargets = <Uri>[];
  late final StreamSubscription<HttpRequest> _subscription;
  var _generation = WarpValidatorGeneration.first;
  var maximumConcurrentRequests = 0;
  var _closed = false;

  Uri get origin =>
      Uri(scheme: 'http', host: _server.address.address, port: _server.port);

  Uri get mediaUrl => origin.replace(path: '/rotating.mp4');
  Uri get stableUrl => origin.replace(path: '/stable.mp4');
  Uint8List get firstBytes => _firstBytes;
  Uint8List get secondBytes => _secondBytes;
  String get firstValidator => _firstValidator;
  String get secondValidator => _secondValidator;
  bool get hasHeldFirstRequest => requests.any((item) => item.isHeld);
  int get activeRequestCount =>
      requests.where((item) => !item.isTerminal).length;
  int get totalRequestCount => requests.length + redirectTargets.length;

  void rotate() => _generation = WarpValidatorGeneration.second;

  void releaseFirstGeneration() {
    if (!_releaseFirst.isCompleted) _releaseFirst.complete();
  }

  Future<void> close() async {
    if (_closed) return;
    _closed = true;
    releaseFirstGeneration();
    await _subscription.cancel();
    await _server.close(force: true);
  }

  Future<void> _dispatch(HttpRequest request) async {
    final generation = switch (request.uri.path) {
      '/generation-a.mp4' => WarpValidatorGeneration.first,
      '/generation-b.mp4' => WarpValidatorGeneration.second,
      '/stable.mp4' => WarpValidatorGeneration.stable,
      _ => null,
    };
    if (generation != null) return _serveRotationRequest(request, generation);
    if (request.uri.path == '/rotating.mp4') return _redirect(request);
    request.response.statusCode = HttpStatus.notFound;
    await request.response.close();
  }

  Future<void> _redirect(HttpRequest request) async {
    final path = _generation == WarpValidatorGeneration.first
        ? '/generation-a.mp4'
        : '/generation-b.mp4';
    final target = origin.replace(path: path);
    redirectTargets.add(target);
    request.response.statusCode = HttpStatus.temporaryRedirect;
    request.response.headers.set(HttpHeaders.locationHeader, target);
    await request.response.close();
  }
}
