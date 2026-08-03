enum VideoMetricObservation { unobserved, observed }

final class VideoMetricObservations {
  const VideoMetricObservations({
    this.likes = VideoMetricObservation.unobserved,
    this.comments = VideoMetricObservation.unobserved,
  });

  final VideoMetricObservation likes;
  final VideoMetricObservation comments;

  VideoMetricObservations applying(VideoMetricObservationUpdate update) {
    return VideoMetricObservations(
      likes: update.likes ?? likes,
      comments: update.comments ?? comments,
    );
  }
}

final class VideoMetricObservationUpdate {
  const VideoMetricObservationUpdate({this.likes, this.comments});

  final VideoMetricObservation? likes;
  final VideoMetricObservation? comments;
}

final class VideoInteractionUpdate {
  const VideoInteractionUpdate({
    required this.likeCount,
    required this.viewerHasLiked,
    this.commentCount,
    this.observations = const VideoMetricObservationUpdate(),
  });

  final int likeCount;
  final bool viewerHasLiked;
  final int? commentCount;
  final VideoMetricObservationUpdate observations;
}

final class VideoPostMetrics {
  factory VideoPostMetrics({
    required int likeCount,
    required int commentCount,
    required bool viewerHasLiked,
    VideoMetricObservations observations = const VideoMetricObservations(),
  }) {
    _checkCount(likeCount, 'likeCount');
    _checkCount(commentCount, 'commentCount');
    return VideoPostMetrics._(
      likeCount,
      commentCount,
      viewerHasLiked,
      observations,
    );
  }

  const VideoPostMetrics._(
    this.likeCount,
    this.commentCount,
    this.viewerHasLiked,
    this.observations,
  );

  final int likeCount;
  final int commentCount;
  final bool viewerHasLiked;
  final VideoMetricObservations observations;

  VideoMetricObservation get likeObservation => observations.likes;
  VideoMetricObservation get commentObservation => observations.comments;
}

void _checkCount(int count, String name) {
  if (count < 0) throw RangeError.value(count, name, 'Cannot be negative.');
}
