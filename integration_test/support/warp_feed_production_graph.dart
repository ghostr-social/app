import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/app/production_video_playback.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/platform/media/ffi_feed_focus_port.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ndk/ndk.dart';
import 'progressive_device_telemetry.dart';
import 'warp_feed_focus_probe.dart';
import 'warp_feed_player_stage_probe.dart';
import 'warp_feed_preparation_probe.dart';
import 'warp_feed_rust_probe.dart';

part 'warp_feed_production_graph_models.dart';

final class WarpFeedProductionGraph {
  const WarpFeedProductionGraph(this._runtime, this._evidence);

  final _WarpFeedRuntime _runtime;
  final _WarpFeedEvidence _evidence;

  AppDependencies get dependencies => _runtime.dependencies;
  ProductionVideoDelivery get delivery => _runtime.delivery;
  Ndk get ndk => _runtime.ndk;
  AppControllerFactory get controllers => _runtime.controllers;
  FeedCubit get cubit => _runtime.cubit;
  VideoPlaybackPort get playback => _runtime.playback;
  ProgressiveDeviceTelemetry get telemetry => _evidence.telemetry;
  WarpFeedPlayerStageProbe get playerStages => _evidence.playerStages;
  WarpFeedPreparationMetrics get preparation => _evidence.preparation;
  WarpFeedFocusProbe get focus => _evidence.focus;
  WarpFeedRustProbe get rustProbe => _evidence.rustProbe;

  VideoFeedRepository get feedRepository =>
      dependencies.videoCatalogServices.feed;

  Future<void> close() async {
    try {
      await cubit.close();
      await dependencies.close();
    } finally {
      await delivery.dispose();
      await ndk.destroy();
    }
  }
}

WarpFeedProductionGraph composeWarpFeedProductionGraph(
  WarpFeedProductionComposition input,
) {
  final focus = WarpFeedFocusProbe(FfiFeedFocusPort(), input.telemetry.probe);
  final playerStages = WarpFeedPlayerStageProbe(
    FfiPlayerPreparationFeedbackPort(),
    () => input.telemetry.probe.elapsed,
  );
  final controllers = AppControllerFactory(
    input.dependencies,
    feedFocus: focus,
  );
  return WarpFeedProductionGraph(
    (
      dependencies: input.dependencies,
      delivery: input.delivery,
      ndk: input.ndk,
      controllers: controllers,
      cubit: controllers.feed(),
      playback: buildProductionVideoPlayback(
        input.delivery,
        playbackTelemetry: input.telemetry,
        playerPreparationFeedback: playerStages,
      ),
    ),
    (
      telemetry: input.telemetry,
      playerStages: playerStages,
      preparation: input.preparation,
      focus: focus,
      rustProbe: input.rustProbe,
    ),
  );
}
