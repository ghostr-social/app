import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/features/publish/domain/video_publication.dart';
import 'package:ghostr/features/session/domain/user_session.dart';

abstract interface class VideoPublishingRepository {
  Future<VideoPublication> publish({
    required UserSession session,
    required SelectedMedia media,
    required String caption,
  });
}
