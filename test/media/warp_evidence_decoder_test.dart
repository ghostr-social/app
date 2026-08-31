import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_evidence_models.dart';

void main() {
  test('WARP evidence decoder maps the versioned causal metric page', () {
    final evidence = WarpEvidencePage.parse(_evidenceJson);

    expect(evidence.planPage.latestRetainedRevision, 9);
    expect(evidence.planPage.records.single.decisionSequence, 12);
    expect(evidence.planPage.records.single.networkStatusGeneration, 3);
    expect(
      evidence.planPage.records.single.networkClass,
      WarpNetworkClass.wifi,
    );
    expect(evidence.planPage.records.single.plan.readyReserve.ready, 2);
    expect(
      evidence.planPage.records.single.plan.readyReserve.candidatePostIds,
      ['p1', 'p2', 'p3'],
    );
    expect(evidence.planPage.records.single.plan.readyReserve.candidateStates, [
      WarpReserveCandidateState.ready,
      WarpReserveCandidateState.unprepared,
      WarpReserveCandidateState.ready,
    ]);
    expect(evidence.evaluation.userVisible.startupSessions, 4);
    expect(evidence.evaluation.efficiency.totalBytes, 1000);
    expect(evidence.evaluation.readiness.readyCoverageMs, 6000);
    expect(evidence.evaluation.integrity.hashMismatches, 0);
    final plan = evidence.planPage.records.single;
    expect(plan.coversFocusGeneration(BigInt.from(7)), isTrue);
    expect(plan.coversFocusGeneration(BigInt.from(6)), isFalse);
    expect(plan.coversFocusGeneration(BigInt.from(9)), isFalse);
  });

  test('WARP plan rejects invalid causal focus intervals', () {
    final zero = WarpEvidencePage.parse(
      _evidenceJson.replaceFirst(
        '"focus_generation":8,"focus_covers_from":7',
        '"focus_generation":0,"focus_covers_from":0',
      ),
    ).planPage.records.single;
    final inverted = WarpEvidencePage.parse(
      _evidenceJson.replaceFirst(
        '"focus_generation":8,"focus_covers_from":7',
        '"focus_generation":7,"focus_covers_from":8',
      ),
    ).planPage.records.single;

    expect(zero.coversFocusGeneration(BigInt.zero), isFalse);
    expect(inverted.coversFocusGeneration(BigInt.from(7)), isFalse);
  });

  test('older evidence without a decision sequence stays decodable', () {
    final legacy = _evidenceJson.replaceFirst('"decision_sequence":12,', '');
    final plan = WarpEvidencePage.parse(legacy).planPage.records.single;

    expect(plan.decisionSequence, isNull);
  });

  test('decoder counts a canonical HLS frame as ordered ready', () {
    final reserve = WarpReadyReserve.fromJson({
      'target': 1,
      'ready': 1,
      'structural': 1,
      'protected': 1,
      'recovery_horizon_ms': 500,
      'underflow_risk_bps': 0,
      'ready_coverage_ms': 0,
      'candidates': [
        {'post': 'private-hls', 'kind': 'Hls', 'state': 'HlsReady'},
      ],
    });

    expect(reserve.orderedReady, 1);
    expect(reserve.candidateKinds.single, WarpReserveCandidateKind.hls);
    expect(reserve.candidateStates.single, WarpReserveCandidateState.ready);
  });
}

const _evidenceJson = r'''
{
  "schema_version":1,
  "plan_page":{"oldest_retained_revision":4,"latest_retained_revision":9,"cursor_truncated":false,"has_more":false,"records":[{"revision":9,"decision_sequence":12,"observed_at_ms":100,"current_post_id":"opaque-post","focus_generation":8,"focus_covers_from":7,"network_status_generation":3,"network_class":"Wifi","network_profile_generation":0,"plan":{"allocations":[],"retained":[],"evictions":[],"discovery_demand":"Hold","mode":"Safety","ready_reserve":{"target":2,"ready":2,"structural":2,"protected":1,"recovery_horizon_ms":1500,"underflow_risk_bps":100,"ready_coverage_ms":6000,"candidates":[{"post":"p1","state":{"Ready":{}}},{"post":"p2","state":"Unprepared"},{"post":"p3","state":{"Ready":{}}}]},"next_reserve":"NotApplicable"}}]},
  "evaluation":{
    "user_visible":{"swipe_to_first_frame":{"samples":4,"p50_ms":300,"p95_ms":700,"p99_ms":900},"startup_sessions":4,"startup_failures":0,"startup_failure_rate_bps":0,"stall_events":0,"stall_ms":0,"stall_ratio_bps":0,"first_frame_quality_bps":10000,"watch_weighted_quality_bps":10000,"quality_discontinuities":0},
    "efficiency":{"total_bytes":1000,"useful_watched_bytes":700,"aborted_bytes":100,"duplicate_hedge_bytes":50,"completable_probe_bytes":25,"full_downloads_never_useful":0,"request_count":5,"playable_videos":4,"requests_per_playable_milli":1250,"connection_restarts_avoided_by_promotion":1,"cpu_micros":10,"storage_byte_ms":20},
    "budget":{"instantaneous_violations":0,"observations":5,"long_run_network_target_error_bps":0,"long_run_storage_target_error_bps":0,"shadow_price_stability_bps":10000,"qoe_per_matched_network_micros":1,"qoe_per_matched_storage_micros":2},
    "readiness":{"reserve_underflows":0,"reserve_underflow_ms":0,"observed_ms":6000,"reserve_underflow_frequency_bps":0,"probability_weighted_ready_reserve_millis":2000,"useful_ready_coverage_ms":6000,"on_time_readiness_samples":2,"on_time_readiness_expected_bps":9500,"on_time_readiness_observed_bps":10000,"on_time_readiness_calibration_error_bps":500,"on_time_readiness_calibration_bps":9500,"replenish_after_burst":{"samples":1,"p50_ms":500,"p95_ms":500,"p99_ms":500},"protected_rescue_slot_claims":1,"protected_rescue_slot_uses":1,"protected_rescue_slot_utilization_bps":10000},
    "adaptation":{"origin_change_points":0,"regret_micros":0,"recovery_after_change":{"samples":0,"p50_ms":0,"p95_ms":0,"p99_ms":0},"success_predictions":1,"success_expected_bps":9000,"success_observed_bps":10000,"success_calibration_error_bps":1000,"latency_predictions":1,"latency_p50_coverage_bps":10000,"latency_p95_coverage_bps":10000,"latency_p99_coverage_bps":10000,"quantile_predictions":1,"quantile_coverage_bps":10000,"exploration_bytes":0,"failed_exploration_bytes":0},
    "semantics":{"focus_sessions":4,"rank_displacement":0,"semantic_regret_micros":0,"transport_substitutions":0,"transport_substitution_rate_bps":0,"exposure_by_origin":{"opaque-origin":4}},
    "integrity":{"hash_mismatches":0,"stale_validator_incidents":0,"false_streamability_classifications":0,"metadata_field_calibration_errors":0,"incorrect_range_splices_prevented":0,"parser_limit_rejections":0,"ssrf_redirect_blocks":0}
  }
}
''';
