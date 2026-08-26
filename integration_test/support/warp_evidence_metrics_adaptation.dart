part of 'warp_evidence_models.dart';

typedef WarpReadinessMetrics = ({
  int reserveUnderflows,
  int reserveUnderflowMs,
  int observedMs,
  int reserveUnderflowFrequencyBps,
  int probabilityWeightedReadyReserveMillis,
  int readyCoverageMs,
  int onTimeReadinessSamples,
  int onTimeReadinessExpectedBps,
  int onTimeReadinessObservedBps,
  int onTimeReadinessCalibrationErrorBps,
  int onTimeReadinessCalibrationBps,
  WarpLatencyDistribution replenishAfterBurst,
  int protectedRescueSlotClaims,
  int protectedRescueSlotUses,
  int protectedRescueSlotUtilizationBps,
});

typedef WarpAdaptationMetrics = ({
  int originChangePoints,
  int regretMicros,
  WarpLatencyDistribution recoveryAfterChange,
  int successPredictions,
  int successExpectedBps,
  int successObservedBps,
  int successCalibrationErrorBps,
  int latencyPredictions,
  int latencyP50CoverageBps,
  int latencyP95CoverageBps,
  int latencyP99CoverageBps,
  int quantilePredictions,
  int quantileCoverageBps,
  int explorationBytes,
  int failedExplorationBytes,
});

typedef WarpSemanticsMetrics = ({
  int focusSessions,
  int rankDisplacement,
  int semanticRegretMicros,
  int transportSubstitutions,
  int transportSubstitutionRateBps,
  Map<String, int> exposureByOrigin,
});

typedef WarpIntegrityMetrics = ({
  int hashMismatches,
  int staleValidatorIncidents,
  int falseStreamabilityClassifications,
  int metadataFieldCalibrationErrors,
  int incorrectRangeSplicesPrevented,
  int parserLimitRejections,
  int ssrfRedirectBlocks,
});

WarpReadinessMetrics _warpReadiness(Map<String, Object?> json) => (
  reserveUnderflows: _warpInt(json, 'reserve_underflows'),
  reserveUnderflowMs: _warpInt(json, 'reserve_underflow_ms'),
  observedMs: _warpInt(json, 'observed_ms'),
  reserveUnderflowFrequencyBps: _warpInt(
    json,
    'reserve_underflow_frequency_bps',
  ),
  probabilityWeightedReadyReserveMillis: _warpInt(
    json,
    'probability_weighted_ready_reserve_millis',
  ),
  readyCoverageMs: _warpInt(json, 'useful_ready_coverage_ms'),
  onTimeReadinessSamples: _warpInt(json, 'on_time_readiness_samples'),
  onTimeReadinessExpectedBps: _warpInt(json, 'on_time_readiness_expected_bps'),
  onTimeReadinessObservedBps: _warpInt(json, 'on_time_readiness_observed_bps'),
  onTimeReadinessCalibrationErrorBps: _warpInt(
    json,
    'on_time_readiness_calibration_error_bps',
  ),
  onTimeReadinessCalibrationBps: _warpInt(
    json,
    'on_time_readiness_calibration_bps',
  ),
  replenishAfterBurst: _warpLatency(_warpChild(json, 'replenish_after_burst')),
  protectedRescueSlotClaims: _warpInt(json, 'protected_rescue_slot_claims'),
  protectedRescueSlotUses: _warpInt(json, 'protected_rescue_slot_uses'),
  protectedRescueSlotUtilizationBps: _warpInt(
    json,
    'protected_rescue_slot_utilization_bps',
  ),
);

WarpAdaptationMetrics _warpAdaptation(Map<String, Object?> json) => (
  originChangePoints: _warpInt(json, 'origin_change_points'),
  regretMicros: _warpInt(json, 'regret_micros'),
  recoveryAfterChange: _warpLatency(_warpChild(json, 'recovery_after_change')),
  successPredictions: _warpInt(json, 'success_predictions'),
  successExpectedBps: _warpInt(json, 'success_expected_bps'),
  successObservedBps: _warpInt(json, 'success_observed_bps'),
  successCalibrationErrorBps: _warpInt(json, 'success_calibration_error_bps'),
  latencyPredictions: _warpInt(json, 'latency_predictions'),
  latencyP50CoverageBps: _warpInt(json, 'latency_p50_coverage_bps'),
  latencyP95CoverageBps: _warpInt(json, 'latency_p95_coverage_bps'),
  latencyP99CoverageBps: _warpInt(json, 'latency_p99_coverage_bps'),
  quantilePredictions: _warpInt(json, 'quantile_predictions'),
  quantileCoverageBps: _warpInt(json, 'quantile_coverage_bps'),
  explorationBytes: _warpInt(json, 'exploration_bytes'),
  failedExplorationBytes: _warpInt(json, 'failed_exploration_bytes'),
);

WarpSemanticsMetrics _warpSemantics(Map<String, Object?> json) => (
  focusSessions: _warpInt(json, 'focus_sessions'),
  rankDisplacement: _warpInt(json, 'rank_displacement'),
  semanticRegretMicros: _warpInt(json, 'semantic_regret_micros'),
  transportSubstitutions: _warpInt(json, 'transport_substitutions'),
  transportSubstitutionRateBps: _warpInt(
    json,
    'transport_substitution_rate_bps',
  ),
  exposureByOrigin: _warpIntegerMap(json, 'exposure_by_origin'),
);

WarpIntegrityMetrics _warpIntegrity(Map<String, Object?> json) => (
  hashMismatches: _warpInt(json, 'hash_mismatches'),
  staleValidatorIncidents: _warpInt(json, 'stale_validator_incidents'),
  falseStreamabilityClassifications: _warpInt(
    json,
    'false_streamability_classifications',
  ),
  metadataFieldCalibrationErrors: _warpInt(
    json,
    'metadata_field_calibration_errors',
  ),
  incorrectRangeSplicesPrevented: _warpInt(
    json,
    'incorrect_range_splices_prevented',
  ),
  parserLimitRejections: _warpInt(json, 'parser_limit_rejections'),
  ssrfRedirectBlocks: _warpInt(json, 'ssrf_redirect_blocks'),
);

Map<String, int> _warpIntegerMap(Map<String, Object?> json, String field) {
  final values = _warpChild(json, field);
  final result = <String, int>{};
  for (final entry in values.entries) {
    if (entry.value is! int) {
      throw FormatException('$field must contain integers.');
    }
    result[entry.key] = entry.value! as int;
  }
  return Map.unmodifiable(result);
}
