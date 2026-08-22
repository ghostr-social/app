import 'dart:async';

import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation_updates.dart';

final class ControlledPlaybackPreparationUpdates
    implements PlaybackPreparationUpdates {
  final _plans = StreamController<PlaybackPreparationPlan>.broadcast(
    sync: true,
  );

  @override
  Stream<PlaybackPreparationPlan> watchPreparation() => _plans.stream;

  void publish(PlaybackPreparationPlan plan) => _plans.add(plan);

  void fail(Object error) => _plans.addError(error, StackTrace.current);

  Future<void> close() => _plans.close();
}
