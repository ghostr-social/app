import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation_updates.dart';

final class WarpFeedPreparationProbe implements PlaybackPreparationUpdates {
  const WarpFeedPreparationProbe(this._delegate, this.metrics);

  final PlaybackPreparationUpdates _delegate;
  final WarpFeedPreparationMetrics metrics;

  @override
  Stream<PlaybackPreparationPlan> watchPreparation() {
    return _delegate.watchPreparation().map((plan) {
      metrics.observe(plan);
      return plan;
    });
  }
}

final class WarpFeedPreparationMetrics {
  var maximumReadyDepth = 0;

  void observe(PlaybackPreparationPlan plan) {
    final ready = plan.upcoming.where((asset) {
      return asset.readiness ==
          PlaybackPreparationReadiness.structuralStartable;
    }).length;
    if (ready > maximumReadyDepth) maximumReadyDepth = ready;
  }
}
