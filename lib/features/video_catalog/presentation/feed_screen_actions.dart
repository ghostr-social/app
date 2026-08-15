part of 'feed_screen.dart';

extension _FeedScreenActions on _FeedScreenState {
  FeedCardActions _actions(
    BuildContext context,
    FeedLoaded state,
    VideoPost post,
    VideoShareState sharing,
  ) {
    final cubit = context.read<FeedCubit>();
    return FeedCardActions(
      navigation: FeedCardNavigationActions(
        onOpenProfile: () => widget.bindings.onOpenProfile(post.creator.id),
        onOpenComments: () => _openComments(context, post),
        onOpenHashtag: widget.bindings.onOpenHashtag,
        onFollowCreator: state.canFollow(post.creator.id)
            ? cubit.followCreator
            : null,
      ),
      engagement: FeedCardEngagementActions(
        onToggleLike: cubit.toggleLike,
        onToggleRepost: cubit.canRepost(post) ? cubit.toggleRepost : null,
      ),
      moderation: FeedCardModerationActions(
        onBlockCreator: () => unawaited(cubit.blockCreator(post)),
      ),
      sharing: FeedCardSharingActions(
        onShare: (post, origin) =>
            context.read<VideoShareCubit>().share(post, origin: origin),
        status: _shareStatus(context, post, sharing),
      ),
    );
  }

  FeedCardShareStatus _shareStatus(
    BuildContext context,
    VideoPost post,
    VideoShareState state,
  ) {
    final sharing = context.read<VideoShareCubit>();
    if (!sharing.supports(post)) return FeedCardShareStatus.unavailable;
    if (state case VideoShareInProgress(:final postId)) {
      return postId == post.id
          ? FeedCardShareStatus.downloading
          : FeedCardShareStatus.busy;
    }
    return FeedCardShareStatus.available;
  }

  Future<void> _openComments(BuildContext context, VideoPost post) async {
    _setCommentsOpen(true);
    try {
      await showModalBottomSheet<void>(
        context: context,
        isScrollControlled: true,
        builder: (_) => BlocProvider(
          create: (_) => widget.bindings.createComments(post)..load(),
          child: CommentsSheet(
            onCommentPublished: () => _commentPublished(context, post),
          ),
        ),
      );
    } finally {
      if (mounted) _setCommentsOpen(false);
    }
  }

  void _commentPublished(BuildContext context, VideoPost post) {
    if (!context.mounted) return;
    context.read<FeedCubit>().commentsPublished(post, 1);
  }
}
