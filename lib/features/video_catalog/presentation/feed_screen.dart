import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/comments/presentation/comments_sheet.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen_bindings.dart';
import 'package:ghostr/features/video_catalog/presentation/video_share_feed_scope.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_kind_selector.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_page_view.dart';
import 'package:ghostr/features/video_sharing/presentation/video_share_cubit.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';

export 'feed_screen_bindings.dart';

class FeedScreen extends StatefulWidget {
  const FeedScreen({required this.bindings, super.key});

  final FeedScreenBindings bindings;

  @override
  State<FeedScreen> createState() => _FeedScreenState();
}

class _FeedScreenState extends State<FeedScreen> {
  bool _commentsOpen = false;

  @override
  Widget build(BuildContext context) {
    return VideoShareFeedScope(
      workflow: widget.bindings.shareWorkflow,
      child: BlocListener<FeedCubit, FeedState>(
        listenWhen: _hasNewNotice,
        listener: _showNotice,
        child: BlocBuilder<FeedCubit, FeedState>(builder: _buildFeed),
      ),
    );
  }

  bool _hasNewNotice(FeedState previous, FeedState current) {
    return current is FeedLoaded && current.notice != null;
  }

  void _showNotice(BuildContext context, FeedState state) {
    final message = (state as FeedLoaded).notice!;
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(message)));
    context.read<FeedCubit>().clearNotice();
  }

  Widget _buildFeed(BuildContext context, FeedState state) {
    return FeedKindOverlay(
      selected: state.kind,
      visible: widget.bindings.showFeedKindSelector,
      onSelected: (kind) => unawaited(context.read<FeedCubit>().load(kind)),
      child: _feedContent(context, state),
    );
  }

  Widget _feedContent(BuildContext context, FeedState state) {
    return switch (state) {
      FeedLoading() => const LoadingPanel(label: 'Loading video feed'),
      FeedEmpty() => _emptyFeed(context),
      FeedFailure(message: final message) => _feedError(context, message),
      FeedLoaded() => _feedPages(context, state),
    };
  }

  Widget _feedError(BuildContext context, String message) {
    return AsyncStatePanel(
      icon: Icons.wifi_tethering_error,
      title: 'Feed unavailable',
      message: message,
      actionLabel: 'Retry',
      onAction: context.read<FeedCubit>().retry,
    );
  }

  // The feed hunts on its own while this panel is visible; the action just
  // lets an impatient viewer skip the backoff delay.
  Widget _emptyFeed(BuildContext context) {
    return AsyncStatePanel(
      icon: Icons.travel_explore,
      title: 'Hunting for videos',
      message:
          'The search keeps running — new clips appear the moment a '
          'relay hands them over. Following creators fills this feed faster.',
      actionLabel: 'Search again',
      onAction: context.read<FeedCubit>().retry,
    );
  }

  Widget _feedPages(BuildContext context, FeedLoaded state) {
    return FeedPageView(
      itemCount: state.posts.length,
      initialPage: state.activeIndex,
      onPageChanged: context.read<FeedCubit>().pageChanged,
      itemBuilder: (_, index) => _feedCard(context, state, index),
    );
  }

  Widget _feedCard(BuildContext context, FeedLoaded state, int index) {
    final post = state.posts[index];
    return BlocBuilder<VideoShareCubit, VideoShareState>(
      builder: (context, sharing) => FeedCard(
        key: ValueKey(post.id.value),
        post: post,
        playback: FeedCardPlayback(
          port: widget.bindings.playbackPort,
          isActive:
              widget.bindings.isActive &&
              !_commentsOpen &&
              index == state.activeIndex,
        ),
        actions: _actions(context, state, post, sharing),
      ),
    );
  }

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
    setState(() => _commentsOpen = true);
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
      if (mounted) setState(() => _commentsOpen = false);
    }
  }

  void _commentPublished(BuildContext context, VideoPost post) {
    if (!context.mounted) return;
    context.read<FeedCubit>().commentsPublished(post, 1);
  }
}
