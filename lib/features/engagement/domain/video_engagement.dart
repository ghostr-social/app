class VideoEngagement {
  factory VideoEngagement({
    required int likeCount,
    required bool viewerHasLiked,
  }) {
    if (likeCount < 0) {
      throw RangeError.value(likeCount, 'likeCount', 'Cannot be negative.');
    }
    return VideoEngagement._(likeCount, viewerHasLiked);
  }

  const VideoEngagement._(this.likeCount, this.viewerHasLiked);

  final int likeCount;
  final bool viewerHasLiked;
}
