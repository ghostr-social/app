import 'dart:async';
import 'dart:convert';
import 'dart:developer';
import 'dart:io';
import 'dart:math' show max;
import 'dart:typed_data';

import 'progressive_device_origin_hls_assets.dart';
import 'progressive_mp4_fixture.dart';
import 'progressive_origin_pacing.dart';

export 'progressive_origin_pacing.dart';

part 'progressive_device_origin_response.dart';
part 'progressive_device_origin_detached_response.dart';
part 'progressive_device_origin_hls.dart';
part 'progressive_device_origin_models.dart';
part 'progressive_device_origin_range.dart';
part 'progressive_device_origin_range_semantics.dart';
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
part 'progressive_device_origin_pre_body_gate.dart';
part 'progressive_device_origin_pre_body_gate_control.dart';
part 'progressive_device_origin_send.dart';
part 'progressive_device_origin_queries.dart';
part 'progressive_device_origin_rendezvous_control.dart';
part 'progressive_device_origin_lifecycle.dart';

final class ProgressiveDeviceOrigin {
  ProgressiveDeviceOrigin._(
    this._server,
    this._responseChunkBytes,
    this._pacing,
    this._validator,
    this._availability,
    this._rangeSemantics,
    this._rangeSemanticsById,
  );

  static Future<ProgressiveDeviceOrigin> start({
    int responseChunkBytes = 16 * 1024,
    ProgressiveOriginPacing pacing =
        const ProgressiveOriginPacing.perResponseDelay(Duration.zero),
    ProgressiveOriginValidator validator = ProgressiveOriginValidator.none,
    ProgressiveOriginAvailability availability =
        ProgressiveOriginAvailability.available,
    ProgressiveOriginRangeSemantics rangeSemantics =
        ProgressiveOriginRangeSemantics.coherent,
    Map<String, ProgressiveOriginRangeSemantics> rangeSemanticsById = const {},
    int port = 0,
  }) async {
    if (responseChunkBytes <= 0) throw ArgumentError.value(responseChunkBytes);
    final server = await HttpServer.bind(InternetAddress.loopbackIPv4, port);
    final origin = ProgressiveDeviceOrigin._(
      server,
      responseChunkBytes,
      _ProgressiveOriginPacer(pacing),
      validator,
      availability,
      rangeSemantics,
      Map.unmodifiable(rangeSemanticsById),
    );
    origin._subscription = server.listen(origin._dispatch);
    return origin;
  }

  final HttpServer _server;
  final int _responseChunkBytes;
  final _ProgressiveOriginPacer _pacing;
  final ProgressiveOriginValidator _validator;
  final ProgressiveOriginAvailability _availability;
  final ProgressiveOriginRangeSemantics _rangeSemantics;
  final Map<String, ProgressiveOriginRangeSemantics> _rangeSemanticsById;
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
  ProgressiveOriginPreBodyGate? _preBodyGate;
  var _nextRequestSequence = 0;
  final _requestSequences = <ProgressiveOriginRequest, int>{};
}
