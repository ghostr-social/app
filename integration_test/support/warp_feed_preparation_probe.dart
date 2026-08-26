import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation_updates.dart';

part 'warp_feed_preparation_metrics.dart';
part 'warp_feed_preparation_observation.dart';

typedef WarpFeedPreparationClock = Duration Function();
typedef WarpFeedPreparationSequence = int Function();

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
