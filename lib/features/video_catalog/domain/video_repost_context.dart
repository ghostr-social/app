enum VideoRepostObservation { unobserved, observed }

/// Viewer-specific repost state, kept apart from public like/comment metrics.
final class VideoRepostContext {
  const VideoRepostContext({
    this.viewerHasReposted = false,
    this.observation = VideoRepostObservation.unobserved,
  });

  final bool viewerHasReposted;
  final VideoRepostObservation observation;
}
