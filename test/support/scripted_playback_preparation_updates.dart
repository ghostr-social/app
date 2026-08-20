import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation_updates.dart';

typedef PreparationWatch = Stream<PlaybackPreparationPlan> Function();

final class ScriptedPlaybackPreparationUpdates
    implements PlaybackPreparationUpdates {
  const ScriptedPlaybackPreparationUpdates(this._watch);

  final PreparationWatch _watch;

  @override
  Stream<PlaybackPreparationPlan> watchPreparation() => _watch();
}
