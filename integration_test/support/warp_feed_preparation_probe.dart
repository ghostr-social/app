import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation_updates.dart';

typedef WarpFeedPreparationClock = Duration Function();

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
  WarpFeedPreparationMetrics(this._clock);

  final WarpFeedPreparationClock _clock;
  final _current = <WarpFeedCurrentPreparation>[];
  var maximumStructuralDepth = 0;

  void observe(PlaybackPreparationPlan plan) {
    _recordCurrent(plan);
    final ready = plan.upcoming.where((asset) {
      return asset.readiness ==
          PlaybackPreparationReadiness.structuralStartable;
    }).length;
    if (ready > maximumStructuralDepth) maximumStructuralDepth = ready;
  }

  void _recordCurrent(PlaybackPreparationPlan plan) {
    final deliveryId = plan.currentDeliveryId;
    if (deliveryId == null) return;
    _current.add(
      WarpFeedCurrentPreparation(
        deliveryId,
        plan.current?.readiness ?? PlaybackPreparationReadiness.preparing,
        _clock(),
      ),
    );
  }

  Duration? firstCurrentAt(
    PlaybackDeliveryId deliveryId,
    PlaybackPreparationReadiness readiness,
  ) {
    for (final observation in _current) {
      if (observation.deliveryId == deliveryId &&
          observation.readiness == readiness) {
        return observation.elapsed;
      }
    }
    return null;
  }
}

final class WarpFeedCurrentPreparation {
  const WarpFeedCurrentPreparation(
    this.deliveryId,
    this.readiness,
    this.elapsed,
  );

  final PlaybackDeliveryId deliveryId;
  final PlaybackPreparationReadiness readiness;
  final Duration elapsed;
}
