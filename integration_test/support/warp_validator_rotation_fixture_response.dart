part of 'warp_validator_rotation_fixture.dart';

extension _WarpValidatorRotationResponse on WarpValidatorRotationFixture {
  Future<void> _serveRotationRequest(
    HttpRequest raw,
    WarpValidatorGeneration generation,
  ) async {
    final request = _recordRequest(raw, generation);
    final bytes = _bytesForGeneration(generation);
    final validator = _validatorForGeneration(generation);
    final plan = _responsePlan(request, bytes.length, validator);
    _configureRotationResponse(raw.response, plan, validator);
    request.statusCode = plan.status;
    if (raw.method == 'HEAD') {
      await _finishHead(raw.response, request);
      return;
    }
    await _streamRotationResponse(raw.response, request, bytes, plan);
  }

  WarpValidatorRequest _recordRequest(
    HttpRequest raw,
    WarpValidatorGeneration generation,
  ) {
    final request = WarpValidatorRequest((
      path: raw.uri.path,
      method: raw.method,
      range: raw.headers.value(HttpHeaders.rangeHeader),
      ifRange: raw.headers.value(HttpHeaders.ifRangeHeader),
      generation: generation,
    ));
    requests.add(request);
    final active = activeRequestCount;
    if (active > maximumConcurrentRequests) maximumConcurrentRequests = active;
    return request;
  }

  Future<void> _streamRotationResponse(
    HttpResponse response,
    WarpValidatorRequest request,
    Uint8List bytes,
    _RotationResponsePlan plan,
  ) async {
    if (_holdsFirstResponse(request, plan)) {
      await _streamHeldRotationResponse(response, request, bytes, plan);
      return;
    }
    try {
      await _writeRotationChunk(response, request, bytes, plan);
      await response.close();
      request.outcome = WarpValidatorRequestOutcome.completed;
    } on Object {
      request.outcome = WarpValidatorRequestOutcome.clientCanceled;
    }
  }

  Future<void> _writeRotationChunk(
    HttpResponse response,
    WarpValidatorRequest request,
    Uint8List bytes,
    _RotationResponsePlan plan,
  ) async {
    if (plan.start >= plan.end) return;
    response.add(Uint8List.sublistView(bytes, plan.start, plan.end));
    await response.flush();
    request.servedBytes += plan.end - plan.start;
  }
}

Future<void> _finishHead(
  HttpResponse response,
  WarpValidatorRequest request,
) async {
  await response.close();
  request.outcome = WarpValidatorRequestOutcome.completed;
}

Uint8List _bytesForGeneration(WarpValidatorGeneration generation) {
  return switch (generation) {
    WarpValidatorGeneration.first ||
    WarpValidatorGeneration.stable => WarpValidatorRotationFixture._firstBytes,
    WarpValidatorGeneration.second => WarpValidatorRotationFixture._secondBytes,
  };
}

String _validatorForGeneration(WarpValidatorGeneration generation) {
  return switch (generation) {
    WarpValidatorGeneration.first =>
      WarpValidatorRotationFixture._firstValidator,
    WarpValidatorGeneration.second =>
      WarpValidatorRotationFixture._secondValidator,
    WarpValidatorGeneration.stable => '"warp-stable"',
  };
}
