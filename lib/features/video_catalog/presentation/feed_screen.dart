import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/comments/presentation/comments_sheet.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';

class FeedScreenBindings {
  const FeedScreenBindings({
    required this.onOpenProfile,
    required this.onOpenHashtag,
    required this.playbackPort,
    required this.createComments,
    required this.isActive,
  });

  final ValueChanged<ProfileId> onOpenProfile;
  final ValueChanged<String> onOpenHashtag;
  final VideoPlaybackPort playbackPort;
  final CommentsCubit Function(VideoPost post) createComments;
  final bool isActive;
}

class FeedScreen extends StatefulWidget {
  const FeedScreen({
    required this.bindings,
    super.key,
  });

  final FeedScreenBindings bindings;

  @override
  State<FeedScreen> createState() => _FeedScreenState();
}

class _FeedScreenState extends State<FeedScreen> {
  bool _commentsOpen = false;

  @override
  Widget build(BuildContext context) {
    return BlocListener<FeedCubit, FeedState>(
      listenWhen: _hasNewNotice,
      listener: _showNotice,
      child: BlocBuilder<FeedCubit, FeedState>(builder: _buildFeed),
    );
  }

  bool _hasNewNotice(FeedState previous, FeedState current) {
    return current is FeedLoaded && current.notice != null;
  }

  void _showNotice(BuildContext context, FeedState state) {
    final message = (state as FeedLoaded).notice!;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message)),
    );
    context.read<FeedCubit>().clearNotice();
  }

  Widget _buildFeed(BuildContext context, FeedState state) {
    return _feedContent(context, state);
  }

  Widget _feedContent(BuildContext context, FeedState state) {
    return switch (state) {
      FeedLoading() => const LoadingPanel(label: 'Loading video feed'),
      FeedEmpty() => _emptyFeed(),
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

  Widget _emptyFeed() {
    return const AsyncStatePanel(
      icon: Icons.videocam_off,
      title: 'No videos yet',
      message: 'Follow creators or publish your first clip to start the loop.',
    );
  }

  Widget _feedPages(BuildContext context, FeedLoaded state) {
    return PageView.builder(
      scrollDirection: Axis.vertical,
      itemCount: state.posts.length,
      onPageChanged: context.read<FeedCubit>().pageChanged,
      itemBuilder: (_, index) => _feedCard(context, state, index),
    );
  }

  Widget _feedCard(BuildContext context, FeedLoaded state, int index) {
    final post = state.posts[index];
    return FeedCard(
      post: post,
      playbackPort: widget.bindings.playbackPort,
      isActive: widget.bindings.isActive &&
          !_commentsOpen &&
          index == state.activeIndex,
      actions: FeedCardActions(
        onOpenProfile: () => widget.bindings.onOpenProfile(post.creator.id),
        onOpenHashtag: widget.bindings.onOpenHashtag,
        onToggleLike: context.read<FeedCubit>().toggleLike,
        onOpenComments: () => _openComments(context, post),
      ),
    );
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
