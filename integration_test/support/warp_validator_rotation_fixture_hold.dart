part of 'warp_validator_rotation_fixture.dart';

extension _WarpValidatorRotationHold on WarpValidatorRotationFixture {
  bool _holdsFirstResponse(
    WarpValidatorRequest request,
    _RotationResponsePlan plan,
  ) {
    return _holdFirstGeneration &&
        request.generation == WarpValidatorGeneration.first &&
        plan.end - plan.start > 4096;
  }

  Future<void> _streamHeldRotationResponse(
    HttpResponse response,
    WarpValidatorRequest request,
    Uint8List bytes,
    _RotationResponsePlan plan,
  ) async {
    Socket? socket;
    try {
      socket = await response.detachSocket(writeHeaders: true);
      final split = min(plan.start + 4096, plan.end);
      socket.add(Uint8List.sublistView(bytes, plan.start, split));
      await socket.flush();
      request.servedBytes += split - plan.start;
      request.wasHeld = true;
      request.outcome = WarpValidatorRequestOutcome.held;
      await _releaseFirst.future;
      if (_generation != WarpValidatorGeneration.first) {
        request.outcome = WarpValidatorRequestOutcome.retiredAfterRotation;
        return;
      }
      socket.add(Uint8List.sublistView(bytes, split, plan.end));
      await socket.flush();
      request.servedBytes += plan.end - split;
      request.outcome = WarpValidatorRequestOutcome.completed;
    } on Object {
      request.outcome = WarpValidatorRequestOutcome.clientCanceled;
    } finally {
      socket?.destroy();
    }
  }
}
