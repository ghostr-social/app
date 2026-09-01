part of 'warp_feed_production_graph_build.dart';

Future<WarpFeedProductionGraph> buildWarpFeedProductionGraph(
  ProgressiveDeviceResources resources,
  WarpFeedRelay relay,
  DataUsageLevel dataUsage, {
  VideoPlaybackCapabilities playbackCapabilities =
      VideoPlaybackCapabilities.progressiveOnly,
  HlsPlaybackGatewayPort? hlsPlaybackGateway,
}) {
  return buildWarpFeedProductionGraphForRelay(
    resources,
    relay.uri,
    WarpFeedProductionGraphOptions(
      dataUsage: dataUsage,
      playbackCapabilities: playbackCapabilities,
      hlsPlaybackGateway: hlsPlaybackGateway,
    ),
  );
}

Future<WarpFeedProductionGraph> buildWarpFeedProductionGraphForRelay(
  ProgressiveDeviceResources resources,
  Uri relay,
  WarpFeedProductionGraphOptions options,
) async {
  SharedPreferences.setMockInitialValues(_settings(relay, options.dataUsage));
  final build = _newBuild(options.account);
  try {
    final dependencies = await _buildDependencies(build, resources, options);
    await build.account.activate(build.capture.nostr!);
    return _composeGraph(build, dependencies);
  } on Object {
    await _closeFailedBuild(build);
    rethrow;
  }
}

final class WarpFeedProductionGraphOptions {
  const WarpFeedProductionGraphOptions({
    required this.dataUsage,
    this.account,
    this.playbackCapabilities = VideoPlaybackCapabilities.progressiveOnly,
    this.hlsPlaybackGateway,
    this.deviceIntegrationOrigin,
  });

  final DataUsageLevel dataUsage;
  final WarpFeedNostrAccount? account;
  final VideoPlaybackCapabilities playbackCapabilities;
  final HlsPlaybackGatewayPort? hlsPlaybackGateway;
  final Uri? deviceIntegrationOrigin;
}
