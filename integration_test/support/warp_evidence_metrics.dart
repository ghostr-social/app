part of 'warp_evidence_models.dart';

typedef WarpLatencyDistribution = ({
  int samples,
  int p50Ms,
  int p95Ms,
  int p99Ms,
});

typedef WarpUserVisibleMetrics = ({
  WarpLatencyDistribution swipeToFirstFrame,
  int startupSessions,
  int startupFailures,
  int startupFailureRateBps,
  int stallEvents,
  int stallMs,
  int stallRatioBps,
  int firstFrameQualityBps,
  int watchWeightedQualityBps,
  int qualityDiscontinuities,
});

typedef WarpEfficiencyMetrics = ({
  int totalBytes,
  int usefulWatchedBytes,
  int abortedBytes,
  int duplicateHedgeBytes,
  int completableProbeBytes,
  int fullDownloadsNeverUseful,
  int requestCount,
  int playableVideos,
  int requestsPerPlayableMilli,
  int connectionRestartsAvoidedByPromotion,
  int cpuMicros,
  int storageByteMs,
});

typedef WarpBudgetMetrics = ({
  int instantaneousViolations,
  int observations,
  int longRunNetworkTargetErrorBps,
  int longRunStorageTargetErrorBps,
  int shadowPriceStabilityBps,
  int qoePerMatchedNetworkMicros,
  int qoePerMatchedStorageMicros,
});

WarpLatencyDistribution _warpLatency(Map<String, Object?> json) => (
  samples: _warpInt(json, 'samples'),
  p50Ms: _warpInt(json, 'p50_ms'),
  p95Ms: _warpInt(json, 'p95_ms'),
  p99Ms: _warpInt(json, 'p99_ms'),
);

WarpUserVisibleMetrics _warpUserVisible(Map<String, Object?> json) => (
  swipeToFirstFrame: _warpLatency(_warpChild(json, 'swipe_to_first_frame')),
  startupSessions: _warpInt(json, 'startup_sessions'),
  startupFailures: _warpInt(json, 'startup_failures'),
  startupFailureRateBps: _warpInt(json, 'startup_failure_rate_bps'),
  stallEvents: _warpInt(json, 'stall_events'),
  stallMs: _warpInt(json, 'stall_ms'),
  stallRatioBps: _warpInt(json, 'stall_ratio_bps'),
  firstFrameQualityBps: _warpInt(json, 'first_frame_quality_bps'),
  watchWeightedQualityBps: _warpInt(json, 'watch_weighted_quality_bps'),
  qualityDiscontinuities: _warpInt(json, 'quality_discontinuities'),
);

WarpEfficiencyMetrics _warpEfficiency(Map<String, Object?> json) => (
  totalBytes: _warpInt(json, 'total_bytes'),
  usefulWatchedBytes: _warpInt(json, 'useful_watched_bytes'),
  abortedBytes: _warpInt(json, 'aborted_bytes'),
  duplicateHedgeBytes: _warpInt(json, 'duplicate_hedge_bytes'),
  completableProbeBytes: _warpInt(json, 'completable_probe_bytes'),
  fullDownloadsNeverUseful: _warpInt(json, 'full_downloads_never_useful'),
  requestCount: _warpInt(json, 'request_count'),
  playableVideos: _warpInt(json, 'playable_videos'),
  requestsPerPlayableMilli: _warpInt(json, 'requests_per_playable_milli'),
  connectionRestartsAvoidedByPromotion: _warpInt(
    json,
    'connection_restarts_avoided_by_promotion',
  ),
  cpuMicros: _warpInt(json, 'cpu_micros'),
  storageByteMs: _warpInt(json, 'storage_byte_ms'),
);

WarpBudgetMetrics _warpBudget(Map<String, Object?> json) => (
  instantaneousViolations: _warpInt(json, 'instantaneous_violations'),
  observations: _warpInt(json, 'observations'),
  longRunNetworkTargetErrorBps: _warpInt(
    json,
    'long_run_network_target_error_bps',
  ),
  longRunStorageTargetErrorBps: _warpInt(
    json,
    'long_run_storage_target_error_bps',
  ),
  shadowPriceStabilityBps: _warpInt(json, 'shadow_price_stability_bps'),
  qoePerMatchedNetworkMicros: _warpInt(json, 'qoe_per_matched_network_micros'),
  qoePerMatchedStorageMicros: _warpInt(json, 'qoe_per_matched_storage_micros'),
);
