part of 'warp_validator_rotation_fixture.dart';

enum WarpValidatorGeneration { first, second, stable }

enum WarpValidatorRequestOutcome {
  serving,
  held,
  retiredAfterRotation,
  completed,
  clientCanceled,
}

final class WarpValidatorRequest {
  WarpValidatorRequest(WarpValidatorRequestData data)
    : path = data.path,
      method = data.method,
      range = data.range,
      ifRange = data.ifRange,
      generation = data.generation;

  final String path;
  final String method;
  final String? range;
  final String? ifRange;
  final WarpValidatorGeneration generation;
  var statusCode = 0;
  var servedBytes = 0;
  var wasHeld = false;
  var outcome = WarpValidatorRequestOutcome.serving;

  bool get isHeld => outcome == WarpValidatorRequestOutcome.held;
  bool get isTerminal =>
      outcome == WarpValidatorRequestOutcome.retiredAfterRotation ||
      outcome == WarpValidatorRequestOutcome.completed ||
      outcome == WarpValidatorRequestOutcome.clientCanceled;
}

typedef WarpValidatorRequestData = ({
  String path,
  String method,
  String? range,
  String? ifRange,
  WarpValidatorGeneration generation,
});

typedef _RotationResponsePlan = ({
  int status,
  int start,
  int end,
  String? contentRange,
});
