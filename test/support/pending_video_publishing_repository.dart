import 'dart:async';

import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/features/publish/domain/video_publishing_repository.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class PendingVideoPublishingRepository implements VideoPublishingRepository {
  final Completer<VideoPost> result = Completer<VideoPost>();
  int publishCount = 0;

  @override
  Future<VideoPost> publish({
    required UserSession session,
    required SelectedMedia media,
    required String caption,
  }) {
    publishCount += 1;
    return result.future;
  }
}
