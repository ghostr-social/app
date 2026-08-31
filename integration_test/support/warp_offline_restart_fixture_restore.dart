part of 'warp_offline_restart_fixture.dart';

Future<WarpOfflineRestartFixture> _startOfflineRestore() async {
  final storage = await WarpOfflineRestartStorage.open();
  final manifest = await storage.read();
  final resources = await ProgressiveDeviceResources.startPersistent(
    storage.cache,
    originPort: manifest.originPort,
    originAvailability: ProgressiveOriginAvailability.unavailable,
  );
  try {
    final graph = await _offlineGraph(resources, manifest.relay);
    graph.network.publish(DeliveryNetworkClass.unavailable);
    return WarpOfflineRestartFixture._(
      resources: resources,
      graph: graph,
      storage: storage,
      manifest: manifest,
    );
  } on Object {
    await resources.close();
    rethrow;
  }
}
