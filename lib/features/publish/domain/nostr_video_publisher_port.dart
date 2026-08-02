import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

abstract interface class NostrVideoPublisherPort {
  Future<VideoPost> publish({
    required UserSession session,
    required SelectedMedia media,
    required String caption,
  });
}
