part of 'warp_feed_production_graph_build.dart';

Future<AppDependencies> _buildDependencies(
  _WarpFeedBuild build,
  ProgressiveDeviceResources resources,
  VideoPlaybackCapabilities playbackCapabilities,
  HlsPlaybackGatewayPort? hlsPlaybackGateway,
) {
  final environment = warpFeedProductionEnvironment(
    build.account.ndk,
    resources,
    build.preparation,
    build.capture,
    playbackCapabilities: playbackCapabilities,
    hlsPlaybackGateway: hlsPlaybackGateway,
  );
  return buildProductionDependencies(environment);
}

Future<void> _closeFailedBuild(_WarpFeedBuild build) async {
  final delivery = build.capture.delivery;
  if (delivery == null) {
    await build.capture.network.close();
  } else {
    await delivery.dispose();
  }
  await build.account.ndk.destroy();
}
