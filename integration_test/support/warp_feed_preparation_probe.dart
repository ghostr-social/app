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
  final _assets = <WarpFeedCurrentPreparation>[];
  var maximumStructuralDepth = 0;
  var maximumReadyDepth = 0;

  void observe(PlaybackPreparationPlan plan) {
    _recordAssets(plan);
    final structural = plan.upcoming
        .where((asset) => asset.readiness.isStructurallyStartable)
        .length;
    final ready = plan.upcoming
        .where((asset) => asset.readiness.isPlayerVerified)
        .length;
    if (structural > maximumStructuralDepth) {
      maximumStructuralDepth = structural;
    }
    if (ready > maximumReadyDepth) maximumReadyDepth = ready;
  }

  void _recordAssets(PlaybackPreparationPlan plan) {
    final elapsed = _clock();
    for (final asset in [
      if (plan.current != null) plan.current!,
      ...plan.upcoming,
    ]) {
      _assets.add(
        WarpFeedCurrentPreparation(asset.authority, asset.readiness, elapsed),
      );
    }
  }

  Duration? firstAt(
    PlaybackAssetAuthority authority,
    PlaybackPreparationReadiness readiness,
  ) => _first(_assets, authority, readiness);

  Duration? firstStructurallyStartableAt(PlaybackAssetAuthority authority) {
    for (final observation in _assets) {
      if (observation.authority == authority &&
          observation.readiness.isStructurallyStartable) {
        return observation.elapsed;
      }
    }
    return null;
  }

  Duration? _first(
    List<WarpFeedCurrentPreparation> observations,
    PlaybackAssetAuthority authority,
    PlaybackPreparationReadiness readiness,
  ) {
    for (final observation in observations) {
      if (observation.authority == authority &&
          observation.readiness == readiness) {
        return observation.elapsed;
      }
    }
    return null;
  }
}

final class WarpFeedCurrentPreparation {
  const WarpFeedCurrentPreparation(
    this.authority,
    this.readiness,
    this.elapsed,
  );
  final PlaybackAssetAuthority authority;
  final PlaybackPreparationReadiness readiness;
  final Duration elapsed;
}
