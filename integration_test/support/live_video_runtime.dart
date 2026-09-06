import 'package:flutter/widgets.dart';
import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/app/ghostr_app.dart';
import 'package:ghostr/app/production_video_playback.dart';
import 'package:ghostr/platform/media/ffi_progressive_playback_gateway.dart';

import 'live_focus_probe.dart';
import 'live_gateway_probe.dart';
import 'live_delivery_evidence.dart';
import 'live_delivery_updates.dart';
import 'live_playback_dependencies.dart';
import 'live_video_environment.dart';
import 'live_video_log.dart';
import 'progressive_device_telemetry.dart';

final class LiveVideoRuntime {
  LiveVideoRuntime(this.log) {
    environment = LiveVideoEnvironment(log);
    focus = LiveFocusProbe(telemetry.probe, log);
  }

  final LiveVideoLog log;
  final telemetry = ProgressiveDeviceTelemetry();
  late final deliveryEvidence = LiveDeliveryEvidence(log);
  late final LiveVideoEnvironment environment;
  late final LiveFocusProbe focus;
  AppDependencies? dependencies;

  Future<Widget> start() async {
    log.add('bootstrap_started', {});
    final production = await buildProductionDependencies(environment.build());
    dependencies = livePlaybackDependencies(
      production,
      buildProductionVideoPlayback(
        environment.delivery!,
        playbackTelemetry: telemetry,
        progressiveGateway: FfiProgressivePlaybackGateway(
          resolvePlaybackUrl: LiveGatewayProbe(log).resolve,
        ),
      ),
    );
    log.add('bootstrap_finished', {});
    return GhostrApp(
      dependencies: dependencies!,
      feedFocus: focus,
      deliveryUpdates: LiveDeliveryUpdates(deliveryEvidence, focus),
    );
  }

  Future<void> close() async {
    deliveryEvidence.summarize();
    await dependencies?.close();
    await environment.delivery?.dispose();
  }
}
