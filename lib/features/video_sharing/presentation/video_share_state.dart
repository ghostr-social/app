import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';

sealed class VideoShareState {
  const VideoShareState();
}

final class VideoShareIdle extends VideoShareState {
  const VideoShareIdle();
}

final class VideoShareInProgress extends VideoShareState {
  const VideoShareInProgress(this.postId);

  final VideoPostId postId;
}

final class VideoShareFailed extends VideoShareState {
  const VideoShareFailed(this.postId, this.message);

  final VideoPostId postId;
  final String message;
}
