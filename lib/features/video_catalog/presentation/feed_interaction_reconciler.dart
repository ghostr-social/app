import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

final class FeedInteractionReconciler {
  final _acceptedLikes = <VideoInteractionTarget, _AcceptedLike>{};
  final _commentFloors = <VideoInteractionTarget, int>{};

  List<VideoPost> acceptLike(
    VideoPost accepted,
    List<VideoPost> current,
  ) {
    final target = VideoInteractionTarget.fromPost(accepted);
    final projection = _AcceptedLike(
      count: accepted.likeCount,
      liked: accepted.viewerHasLiked,
    );
    _acceptedLikes[target] = projection;
    return current.map((post) {
      return _target(post) == target ? _withLike(post, projection) : post;
    }).toList(growable: false);
  }

  List<VideoPost> acceptComments(
    VideoPost source,
    int publishedCount,
    List<VideoPost> current,
  ) {
    final target = _target(source);
    final matching =
        current.where((post) => _target(post) == target).firstOrNull;
    final currentCount = matching?.commentCount ?? source.commentCount;
    final baseline = _max(currentCount, _commentFloors[target] ?? 0);
    final floor = baseline + publishedCount;
    _commentFloors[target] = floor;
    return current.map((post) {
      return _target(post) == target ? _withComments(post, floor) : post;
    }).toList(growable: false);
  }

  List<VideoPost> reconcile({
    required List<VideoPost> refreshed,
    required List<VideoPost> current,
  }) {
    final currentByTarget = <VideoInteractionTarget, VideoPost>{
      for (final post in current) _target(post): post,
    };
    return refreshed.map((post) {
      final target = _target(post);
      return _reconcilePost(post, currentByTarget[target], target);
    }).toList(growable: false);
  }

  VideoPost _reconcilePost(
    VideoPost refreshed,
    VideoPost? current,
    VideoInteractionTarget target,
  ) {
    final like = _reconciledLike(refreshed, current, target);
    final comments = _reconciledComments(refreshed, current, target);
    if (_hasInteractions(refreshed, like, comments)) return refreshed;
    return refreshed.withInteraction(
      VideoInteractionUpdate(
        likeCount: like.count,
        viewerHasLiked: like.liked,
        commentCount: comments,
      ),
    );
  }

  bool _hasInteractions(
    VideoPost post,
    _AcceptedLike like,
    int comments,
  ) {
    return post.likeCount == like.count &&
        post.viewerHasLiked == like.liked &&
        post.commentCount == comments;
  }

  _AcceptedLike _reconciledLike(
    VideoPost refreshed,
    VideoPost? current,
    VideoInteractionTarget target,
  ) {
    final accepted = _acceptedLikes[target];
    if (_observedLike(refreshed) && _confirmsLike(refreshed, accepted)) {
      _acceptedLikes.remove(target);
      return _likeFrom(refreshed);
    }
    if (accepted != null) return _projectLike(refreshed, accepted);
    if (!_observedLike(refreshed) && current != null) {
      return _likeFrom(current);
    }
    return _likeFrom(refreshed);
  }

  int _reconciledComments(
    VideoPost refreshed,
    VideoPost? current,
    VideoInteractionTarget target,
  ) {
    final floor = _commentFloors[target];
    if (_observedComments(refreshed) &&
        floor != null &&
        refreshed.commentCount >= floor) {
      _commentFloors.remove(target);
      return refreshed.commentCount;
    }
    if (floor != null) return _max(refreshed.commentCount, floor);
    if (!_observedComments(refreshed) && current != null) {
      return current.commentCount;
    }
    return refreshed.commentCount;
  }

  _AcceptedLike _projectLike(VideoPost refreshed, _AcceptedLike accepted) {
    final adjustment = refreshed.viewerHasLiked == accepted.liked
        ? 0
        : (accepted.liked ? 1 : -1);
    final projected = _max(0, refreshed.likeCount + adjustment);
    return _AcceptedLike(
      count: _max(accepted.count, projected),
      liked: accepted.liked,
    );
  }

  VideoPost _withLike(VideoPost post, _AcceptedLike like) {
    return post.withInteraction(
      VideoInteractionUpdate(
        likeCount: like.count,
        viewerHasLiked: like.liked,
      ),
    );
  }

  VideoPost _withComments(VideoPost post, int count) {
    return post.withInteraction(
      VideoInteractionUpdate(
        likeCount: post.likeCount,
        viewerHasLiked: post.viewerHasLiked,
        commentCount: _max(post.commentCount, count),
      ),
    );
  }

  _AcceptedLike _likeFrom(VideoPost post) {
    return _AcceptedLike(count: post.likeCount, liked: post.viewerHasLiked);
  }

  bool _confirmsLike(VideoPost post, _AcceptedLike? accepted) {
    return accepted == null || post.viewerHasLiked == accepted.liked;
  }

  bool _observedLike(VideoPost post) {
    return post.metrics.likeObservation == VideoMetricObservation.observed;
  }

  bool _observedComments(VideoPost post) {
    return post.metrics.commentObservation == VideoMetricObservation.observed;
  }

  VideoInteractionTarget _target(VideoPost post) {
    return VideoInteractionTarget.fromPost(post);
  }

  int _max(int left, int right) => left > right ? left : right;
}

final class _AcceptedLike {
  const _AcceptedLike({required this.count, required this.liked});

  final int count;
  final bool liked;
}
