part of 'warp_offline_restart_fixture.dart';

Future<WarpOfflineRestartFixture> _startOfflineSeed() async {
  final storage = await WarpOfflineRestartStorage.reset();
  final resources = await ProgressiveDeviceResources.startPersistent(
    storage.cache,
    validator: ProgressiveOriginValidator.stableStrong,
  );
  WarpFeedRelay? relay;
  try {
    final events = await signedWarpFeedEvents(
      resources.origin,
      config: const SignedWarpFeedConfig(eventCount: 1),
    );
    relay = await WarpFeedRelay.start(events);
    final graph = await _offlineGraph(resources, relay.uri);
    final manifest = _offlineManifest(events.single.id, resources, relay);
    await storage.write(manifest);
    return WarpOfflineRestartFixture._(
      resources: resources,
      graph: graph,
      storage: storage,
      manifest: manifest,
      relay: relay,
    );
  } on Object {
    await relay?.close();
    await resources.close();
    await storage.delete();
    rethrow;
  }
}

Future<WarpFeedProductionGraph> _offlineGraph(
  ProgressiveDeviceResources resources,
  Uri relay,
) {
  return buildWarpFeedProductionGraphForRelay(
    resources,
    relay,
    WarpFeedProductionGraphOptions(
      dataUsage: DataUsageLevel.aggressive,
      account: warpOfflineRestartAccount(),
    ),
  );
}

WarpOfflineRestartManifest _offlineManifest(
  String eventId,
  ProgressiveDeviceResources resources,
  WarpFeedRelay relay,
) {
  return WarpOfflineRestartManifest(
    eventId: eventId,
    originPort: resources.origin.origin.port,
    relay: relay.uri,
    viewerPublicKey: warpOfflineRestartPublicKey(),
  );
}
