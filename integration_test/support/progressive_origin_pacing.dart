final class ProgressiveOriginPacing {
  const ProgressiveOriginPacing.perResponseDelay(this.responseChunkDelay)
    : bandwidthKbps = null;

  const ProgressiveOriginPacing.sharedBandwidth(this.bandwidthKbps)
    : responseChunkDelay = Duration.zero,
      assert(bandwidthKbps != null && bandwidthKbps > 0);

  final Duration responseChunkDelay;
  final int? bandwidthKbps;

  bool get isShared => bandwidthKbps != null;
}
