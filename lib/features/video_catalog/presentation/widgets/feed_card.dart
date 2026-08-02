import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class FeedCardActions {
  const FeedCardActions({
    required this.onOpenProfile,
    required this.onToggleLike,
    required this.onOpenComments,
  });

  final VoidCallback onOpenProfile;
  final Future<void> Function(VideoPost post) onToggleLike;
  final VoidCallback onOpenComments;
}

class FeedCard extends StatefulWidget {
  const FeedCard({
    required this.post,
    required this.playbackPort,
    required this.isActive,
    required this.actions,
    super.key,
  });

  final VideoPost post;
  final VideoPlaybackPort playbackPort;
  final bool isActive;
  final FeedCardActions actions;

  @override
  State<FeedCard> createState() => _FeedCardState();
}

class _FeedCardState extends State<FeedCard> {
  bool _isTogglingLike = false;

  @override
  Widget build(BuildContext context) {
    return Stack(
      fit: StackFit.expand,
      children: [
        _videoSurface(),
        _scrim(),
        _content(context),
      ],
    );
  }

  Widget _videoSurface() {
    return widget.playbackPort.buildSurface(
      media: widget.post.media,
      isActive: widget.isActive,
    );
  }

  Widget _scrim() {
    return DecoratedBox(
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [
            AppPalette.videoScrimTop,
            AppPalette.videoScrimBottom,
          ],
        ),
      ),
    );
  }

  Widget _content(BuildContext context) {
    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.all(AppSpacing.lg),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _profileButton(),
            const Spacer(),
            _metadata(context),
            const SizedBox(height: AppSpacing.lg),
            _interactionChips(),
          ],
        ),
      ),
    );
  }

  Widget _profileButton() {
    return Align(
      alignment: Alignment.topRight,
      child: FilledButton.tonal(
        onPressed: widget.actions.onOpenProfile,
        child: const Text('Profile'),
      ),
    );
  }

  Widget _metadata(BuildContext context) {
    final muted = Theme.of(context).textTheme.bodyLarge?.copyWith(
          color: AppPalette.mutedForeground,
        );
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(widget.post.creator.displayName,
            style: Theme.of(context).textTheme.headlineMedium),
        const SizedBox(height: AppSpacing.xxs),
        Text(widget.post.creator.handle, style: muted),
        const SizedBox(height: AppSpacing.sm),
        Text(widget.post.caption,
            style: Theme.of(context).textTheme.titleMedium),
        const SizedBox(height: AppSpacing.xs),
        Text(widget.post.songName, style: muted),
      ],
    );
  }

  Widget _interactionChips() {
    return Wrap(
      spacing: AppSpacing.xs,
      runSpacing: AppSpacing.xs,
      children: [_likeChip(), _commentChip()],
    );
  }

  Widget _likeChip() {
    final isLiked = widget.post.viewerHasLiked;
    return ActionChip(
      avatar: Icon(isLiked ? Icons.favorite : Icons.favorite_border),
      label: Text('${widget.post.likeCount} likes'),
      tooltip: isLiked ? 'Unlike video' : 'Like video',
      onPressed: _isTogglingLike ? null : _toggleLike,
    );
  }

  Widget _commentChip() {
    return ActionChip(
      avatar: const Icon(Icons.chat_bubble_outline),
      label: Text('${widget.post.commentCount} comments'),
      tooltip: 'Open comments',
      onPressed: widget.actions.onOpenComments,
    );
  }

  Future<void> _toggleLike() async {
    setState(() => _isTogglingLike = true);
    try {
      await widget.actions.onToggleLike(widget.post);
    } finally {
      if (mounted) setState(() => _isTogglingLike = false);
    }
  }
}
