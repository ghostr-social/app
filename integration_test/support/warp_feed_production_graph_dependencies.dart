part of 'warp_feed_production_graph_build.dart';

Future<AppDependencies> _buildDependencies(
  _WarpFeedBuild build,
  ProgressiveDeviceResources resources,
  WarpFeedProductionGraphOptions options,
) {
  final environment = warpFeedProductionEnvironment((
    ndk: build.account.ndk,
    resources: resources,
    preparation: build.preparation,
    capture: build.capture,
    playbackCapabilities: options.playbackCapabilities,
    hlsPlaybackGateway: options.hlsPlaybackGateway,
    deviceIntegrationOrigin: options.deviceIntegrationOrigin,
  ));
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
