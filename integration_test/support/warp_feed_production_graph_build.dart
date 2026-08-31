import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_gateway_port.dart';
import 'package:ghostr/platform/media/ffi_playback_preparation_updates.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'progressive_device_resources.dart';
import 'progressive_device_telemetry.dart';
import 'warp_feed_nostr_account.dart';
import 'warp_feed_preparation_probe.dart';
import 'warp_feed_production_environment.dart';
import 'warp_feed_production_graph.dart';
import 'warp_feed_relay.dart';

part 'warp_feed_production_graph_dependencies.dart';
part 'warp_feed_production_graph_relay.dart';

typedef _WarpFeedBuild = ({
  WarpFeedNostrAccount account,
  ProgressiveDeviceTelemetry telemetry,
  WarpFeedPreparationMetrics metrics,
  WarpFeedPreparationProbe preparation,
  WarpFeedProductionCapture capture,
});

_WarpFeedBuild _newBuild(WarpFeedNostrAccount? providedAccount) {
  final account = providedAccount ?? WarpFeedNostrAccount.create();
  final telemetry = ProgressiveDeviceTelemetry();
  final metrics = WarpFeedPreparationMetrics(
    () => telemetry.probe.elapsed,
    telemetry.probe.markExternalEvidence,
  );
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
    network: build.capture.network,
  ));
}

Map<String, Object> _settings(Uri relay, DataUsageLevel dataUsage) => {
  'ghostr.settings.relays': <String>[relay.toString()],
  'ghostr.settings.searchRelays': <String>[],
  'ghostr.settings.dataUsage': dataUsage.name,
};
