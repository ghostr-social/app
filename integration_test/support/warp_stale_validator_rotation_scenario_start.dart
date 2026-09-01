part of 'warp_stale_validator_rotation_scenario.dart';

Future<_WarpValidatorRotationScenario> _startWithFixture(
  WarpValidatorRotationFixture fixture,
) async {
  final resources = await ProgressiveDeviceResources.start();
  try {
    return await _startWithResources(fixture, resources);
  } on Object {
    await resources.close();
    rethrow;
  }
}

Future<_WarpValidatorRotationScenario> _startWithResources(
  WarpValidatorRotationFixture fixture,
  ProgressiveDeviceResources resources,
) async {
  final events = await signedWarpValidatorRotationEvents(fixture);
  final relay = await WarpFeedRelay.start(events);
  try {
    final graph = await buildWarpFeedProductionGraphForRelay(
      resources,
      relay.uri,
      WarpFeedProductionGraphOptions(
        dataUsage: DataUsageLevel.aggressive,
        deviceIntegrationOrigin: fixture.origin,
      ),
    );
    return _WarpValidatorRotationScenario((
      fixture: fixture,
      resources: resources,
      relay: relay,
      events: events,
      graph: graph,
    ));
  } on Object {
    await relay.close();
    rethrow;
  }
}
