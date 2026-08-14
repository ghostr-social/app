import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_context.dart';

final class FeedRepostReconciler {
  final _accepted = <VideoInteractionTarget, bool>{};

  List<VideoPost> accept(VideoPost accepted, List<VideoPost> current) {
    final target = VideoInteractionTarget.fromPost(accepted);
    _accepted[target] = accepted.viewerHasReposted;
    return project(accepted, current);
  }

  List<VideoPost> project(VideoPost projection, List<VideoPost> current) {
    final target = VideoInteractionTarget.fromPost(projection);
    return current
        .map((post) {
          return VideoInteractionTarget.fromPost(post) == target
              ? post.withRepost(projection.viewerHasReposted)
              : post;
        })
        .toList(growable: false);
  }

  List<VideoPost> reconcile(
    List<VideoPost> refreshed,
    List<VideoPost> current,
  ) {
    final currentByTarget = <VideoInteractionTarget, VideoPost>{
      for (final post in current) VideoInteractionTarget.fromPost(post): post,
    };
    return refreshed
        .map((post) {
          final target = VideoInteractionTarget.fromPost(post);
          return _reconciled(post, currentByTarget[target], target);
        })
        .toList(growable: false);
  }

  List<VideoPost> settled(List<VideoPost> hydrated, List<VideoPost> current) {
    final states = <VideoInteractionTarget, bool>{};
    for (final post in hydrated) {
      final target = VideoInteractionTarget.fromPost(post);
      if (_observed(post) && !_accepted.containsKey(target)) {
        states[target] = post.viewerHasReposted;
      }
    }
    return current
        .map((post) => _settledPost(post, states))
        .toList(growable: false);
  }

  VideoPost _settledPost(
    VideoPost post,
    Map<VideoInteractionTarget, bool> states,
  ) {
    if (_observed(post)) return post;
    final state = states[VideoInteractionTarget.fromPost(post)];
    return state == null
        ? post
        : post.withRepost(state, observation: VideoRepostObservation.observed);
  }

  VideoPost _reconciled(
    VideoPost refreshed,
    VideoPost? current,
    VideoInteractionTarget target,
  ) {
    final accepted = _accepted[target];
    if (_observed(refreshed)) {
      _accepted.remove(target);
      return refreshed;
    }
    if (accepted != null) return refreshed.withRepost(accepted);
    if (!_observed(refreshed) && current != null) {
      return refreshed.withRepost(current.viewerHasReposted);
    }
    return refreshed;
  }

  bool _observed(VideoPost post) {
    return post.repostContext.observation == VideoRepostObservation.observed;
  }
}
