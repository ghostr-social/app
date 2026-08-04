import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_like_policy.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_failure_messages.dart';

/// A like as the relays settled it: the post to show, plus the reason when
/// the relays refused and the tap has to be taken back.
final class FeedLike {
  const FeedLike(this.post, {this.message});

  final VideoPost post;
  final String? message;
}

/// What blocking a creator means for the feed.
sealed class FeedBlock {
  const FeedBlock();
}

/// The creator is blocked; their posts must leave the feed.
final class FeedCreatorBlocked extends FeedBlock {
  const FeedCreatorBlocked();
}

/// Nothing to remove: the tap unblocked the creator, or this feed has no
/// social graph to block with.
final class FeedCreatorKept extends FeedBlock {
  const FeedCreatorKept();
}

/// The relays could not be told; the reason is for the viewer.
final class FeedBlockFailed extends FeedBlock {
  const FeedBlockFailed(this.message);

  final String message;
}

/// Runs the viewer's engagement taps against the relays and turns every
/// failure into a reason the feed can show.
final class FeedEngagement {
  const FeedEngagement(this._engagement, [this._social]);

  static const _likes = VideoLikePolicy();

  final VideoEngagementRepository _engagement;
  final SocialGraphRepository? _social;

  /// The post as it looks the instant the heart is tapped.
  VideoPost optimistic(VideoPost post) => _likes.toggle(post);

  /// Confirms the like with the relays. A refusal comes back as the original
  /// post so the heart can flip back.
  Future<FeedLike> confirmLike(VideoPost post) async {
    try {
      return FeedLike(await _engagement.toggleLike(post));
    } on AppFailure catch (failure) {
      return FeedLike(post, message: failure.message);
    } on Object catch (error, stackTrace) {
      return FeedLike(
        post,
        message: unexpectedFeedLikeMessage(error, stackTrace),
      );
    }
  }

  /// Toggles the block on this post's creator.
  Future<FeedBlock> block(VideoPost post) async {
    final social = _social;
    if (social == null) return const FeedCreatorKept();
    try {
      final isBlocked = await social.toggleBlock(post.creator.id);
      return isBlocked ? const FeedCreatorBlocked() : const FeedCreatorKept();
    } on AppFailure catch (failure) {
      return FeedBlockFailed(failure.message);
    } on Object catch (error, stackTrace) {
      return FeedBlockFailed(unexpectedFeedBlockMessage(error, stackTrace));
    }
  }
}
