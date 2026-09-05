import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';

FfiPlayerPreparationFeedbackPort preparationFeedback(
  RustPlayerPreparationReporter reporter,
) => FfiPlayerPreparationFeedbackPort(
  reportPreparation: reporter,
  playerCapabilityGeneration: BigInt.one,
  clientEpoch: BigInt.one,
  monotonicMicros: () => 1,
);
