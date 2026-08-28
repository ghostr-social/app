import 'dart:async';
import 'dart:developer';
import 'dart:io';
import 'dart:math' show max;
import 'dart:typed_data';

import 'progressive_mp4_fixture.dart';
import 'progressive_origin_pacing.dart';

export 'progressive_origin_pacing.dart';

part 'progressive_device_origin_response.dart';
part 'progressive_device_origin_range.dart';
part 'progressive_device_origin_request.dart';
part 'progressive_device_origin_body_queries.dart';
part 'progressive_device_origin_coverage.dart';
part 'progressive_device_origin_concurrency.dart';
part 'progressive_device_origin_rendezvous.dart';
part 'progressive_device_origin_rendezvous_validation.dart';
part 'progressive_device_origin_link_models.dart';
part 'progressive_device_origin_link_profile.dart';
part 'progressive_device_origin_link.dart';
part 'progressive_device_origin_link_control.dart';
part 'progressive_device_origin_link_queries.dart';
part 'progressive_device_origin_bandwidth_trigger.dart';
part 'progressive_device_origin_bandwidth_trigger_control.dart';
part 'progressive_device_origin_chunk_gate.dart';
part 'progressive_device_origin_chunk_gate_control.dart';
part 'progressive_device_origin_send.dart';

part 'progressive_device_origin_rendezvous_validation.dart';

enum ProgressiveOriginValidator { none, stableStrong }

final class ProgressiveDeviceOrigin {
  ProgressiveDeviceOrigin._(
    this._server,
    this._responseChunkBytes,
    this._pacing,
    this._validator,
  );

  static Future<ProgressiveDeviceOrigin> start({
    int responseChunkBytes = 16 * 1024,
    ProgressiveOriginPacing pacing =
        const ProgressiveOriginPacing.perResponseDelay(Duration.zero),
    ProgressiveOriginValidator validator = ProgressiveOriginValidator.none,
  }) async {
    if (responseChunkBytes <= 0) throw ArgumentError.value(responseChunkBytes);
    final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    final origin = ProgressiveDeviceOrigin._(
      server,
      responseChunkBytes,
      _ProgressiveOriginPacer(pacing),
      validator,
    );
    origin._subscription = server.listen(origin._dispatch);
    return origin;
  }

  final HttpServer _server;
  final int _responseChunkBytes;
  final _ProgressiveOriginPacer _pacing;
  final ProgressiveOriginValidator _validator;
  final requests = <ProgressiveOriginRequest>[];
  final _completed = <ProgressiveOriginRequest>[];
  final _heldHeads = <HttpResponse>[];
  final _servedBytes = <String, int>{};
  final _clock = Stopwatch()..start();
  late final StreamSubscription<HttpRequest> _subscription;
  final _concurrency = _ProgressiveOriginConcurrency();
  ProgressiveOriginFirstChunkRendezvous? _firstChunkRendezvous;
  ProgressiveOriginBandwidthTrigger? _bandwidthTrigger;
  ProgressiveOriginChunkGate? _chunkGate;
  var _nextRequestSequence = 0;
  final _requestSequences = <ProgressiveOriginRequest, int>{};

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

  ProgressiveOriginFirstChunkRendezvous rendezvousFirstChunks(
    Set<String> paths, {
    Duration timeout = const Duration(seconds: 2),
  }) => _installRendezvous(
    paths,
    timeout,
    activated: true,
    blocksFirstChunks: true,
  );

  ProgressiveOriginFirstChunkRendezvous stageFirstChunks(
    Set<String> paths, {
    Duration timeout = const Duration(seconds: 2),
  }) => _installRendezvous(
    paths,
    timeout,
    activated: false,
    blocksFirstChunks: false,
  );

  ProgressiveOriginFirstChunkRendezvous _installRendezvous(
    Set<String> paths,
    Duration timeout, {
    required bool activated,
    required bool blocksFirstChunks,
  }) {
    final active = _firstChunkRendezvous;
    if (active != null && !active.isReleased) {
      throw StateError('A first-chunk rendezvous is already active.');
    }
    final rendezvous = ProgressiveOriginFirstChunkRendezvous._(
      paths,
      timeout,
      activated,
      blocksFirstChunks,
    );
    _firstChunkRendezvous = rendezvous;
    return rendezvous;
  }

  List<({int start, int end})> rangesFor(String id) => _completed
      .where((request) => request.path == '/$id.mp4')
      .map((request) => request.range)
      .whereType<({int start, int end})>()
      .toList();

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
      startedAt: _clock.elapsed,
    );
    requests.add(entry);
    _requestSequences[entry] = ++_nextRequestSequence;
    if (request.method == 'HEAD') {
      entry._blockHead();
      _heldHeads.add(request.response);
      return;
    }
    final completed = await _write(request.response, range, entry);
    if (completed) _completed.add(entry);
  }

  void _recordBytes(ProgressiveOriginRequest request, int count) {
    request._recordBytes(count, _clock.elapsed);
    _servedBytes.update(
      request.path,
      (total) => total + count,
      ifAbsent: () => count,
    );
  }

  Future<void> close() async {
    _firstChunkRendezvous?.release();
    _bandwidthTrigger?.cancel();
    _chunkGate?.release();
    for (final response in _heldHeads) {
      await response.close();
    }
    await _subscription.cancel();
    await _server.close(force: true);
  }
}
