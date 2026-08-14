import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_operation_failure.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_context.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_policy.dart';

final class FeedRepost {
  const FeedRepost(this.post, {this.failure});

  final VideoPost post;
  final FeedOperationFailure? failure;
}

final class FeedReposts {
  FeedReposts(this._repository);

  static const _policy = VideoRepostPolicy();
  final VideoRepostRepository _repository;
  final _inFlight = <VideoInteractionTarget>{};
  final _settled = <VideoInteractionTarget>{};
  var _generation = 0;

  bool supports(VideoPost post) => _policy.supports(post);

  VideoPost optimistic(VideoPost post) => _policy.toggle(post);

  Future<FeedRepost> confirm(VideoPost post) async {
    try {
      return FeedRepost(await _repository.toggleRepost(post));
    } on Object catch (error, stackTrace) {
      return FeedRepost(post, failure: FeedOperationFailure(error, stackTrace));
    }
  }

  Future<List<VideoPost>> settle(List<VideoPost> posts) async {
    final generation = _generation;
    final pending = _pending(posts);
    if (pending.isEmpty) return const <VideoPost>[];
    try {
      final settled = await _repository.hydrateAll(
        pending,
        mode: VideoRepostHydration.patient,
      );
      if (generation != _generation) return const <VideoPost>[];
      _release(pending);
      final observed = _observedPosts(settled);
      _settled.addAll(observed.map(VideoInteractionTarget.fromPost));
      return observed;
    } on Object {
      if (generation == _generation) _release(pending);
      return const <VideoPost>[];
    }
  }

  void forget() {
    _generation += 1;
    _inFlight.clear();
    _settled.clear();
  }

  List<VideoPost> _pending(List<VideoPost> posts) {
    final pending = <VideoPost>[];
    for (final post in posts) {
      if (!_needsHydration(post)) continue;
      final target = VideoInteractionTarget.fromPost(post);
      if (_reserve(target)) pending.add(post);
    }
    return pending;
  }

  bool _needsHydration(VideoPost post) {
    return post.repostContext.observation ==
            VideoRepostObservation.unobserved &&
        supports(post);
  }

  bool _reserve(VideoInteractionTarget target) {
    return !_settled.contains(target) && _inFlight.add(target);
  }

  List<VideoPost> _observedPosts(List<VideoPost> posts) {
    return posts
        .where(
          (post) =>
              post.repostContext.observation == VideoRepostObservation.observed,
        )
        .toList(growable: false);
  }

  void _release(List<VideoPost> posts) {
    _inFlight.removeAll(posts.map(VideoInteractionTarget.fromPost));
  }
}
