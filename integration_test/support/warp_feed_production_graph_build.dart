import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/platform/media/ffi_playback_preparation_updates.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'progressive_device_resources.dart';
import 'progressive_device_telemetry.dart';
import 'warp_feed_nostr_account.dart';
import 'warp_feed_preparation_probe.dart';
import 'warp_feed_production_environment.dart';
import 'warp_feed_production_graph.dart';
import 'warp_feed_relay.dart';

Future<WarpFeedProductionGraph> buildWarpFeedProductionGraph(
  ProgressiveDeviceResources resources,
  WarpFeedRelay relay,
) async {
  SharedPreferences.setMockInitialValues(_settings(relay));
  final build = _newBuild();
  try {
    final dependencies = await _buildDependencies(build, resources);
    await build.account.activate(build.capture.nostr!);
    return _composeGraph(build, dependencies);
  } on Object {
    await _closeFailedBuild(build);
    rethrow;
  }
}

typedef _WarpFeedBuild = ({
  WarpFeedNostrAccount account,
  ProgressiveDeviceTelemetry telemetry,
  WarpFeedPreparationMetrics metrics,
  WarpFeedPreparationProbe preparation,
  WarpFeedProductionCapture capture,
});

_WarpFeedBuild _newBuild() {
  final account = WarpFeedNostrAccount.create();
  final telemetry = ProgressiveDeviceTelemetry();
  final metrics = WarpFeedPreparationMetrics(() => telemetry.probe.elapsed);
  return (
    account: account,
    telemetry: telemetry,
    metrics: metrics,
    preparation: WarpFeedPreparationProbe(
      const FfiPlaybackPreparationUpdates(),
      metrics,
    ),
    capture: WarpFeedProductionCapture(),
  );
}

Future<AppDependencies> _buildDependencies(
  _WarpFeedBuild build,
  ProgressiveDeviceResources resources,
) {
  final environment = warpFeedProductionEnvironment(
    build.account.ndk,
    resources,
    build.preparation,
    build.capture,
  );
  return buildProductionDependencies(environment);
}

WarpFeedProductionGraph _composeGraph(
  _WarpFeedBuild build,
  AppDependencies dependencies,
) {
  return composeWarpFeedProductionGraph((
    dependencies: dependencies,
    delivery: build.capture.delivery!,
    ndk: build.account.ndk,
    telemetry: build.telemetry,
    preparation: build.metrics,
    rustProbe: build.capture.rustProbe,
  ));
}

Future<void> _closeFailedBuild(_WarpFeedBuild build) async {
  await build.capture.delivery?.dispose();
  await build.account.ndk.destroy();
}

Map<String, Object> _settings(WarpFeedRelay relay) => {
  'ghostr.settings.relays': <String>[relay.uri.toString()],
  'ghostr.settings.searchRelays': <String>[],
};
