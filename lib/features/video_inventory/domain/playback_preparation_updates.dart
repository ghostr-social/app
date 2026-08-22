import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

abstract interface class PlaybackPreparationUpdates {
  Stream<PlaybackPreparationPlan> watchPreparation();
}
