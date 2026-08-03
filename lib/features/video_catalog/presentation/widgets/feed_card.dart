import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/caption_text.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/profile_avatar.dart';

class FeedCardActions {
  const FeedCardActions({
    required this.onOpenProfile,
    required this.onToggleLike,
    required this.onOpenComments,
    required this.onOpenHashtag,
  });

  final VoidCallback onOpenProfile;
  final Future<void> Function(VideoPost post) onToggleLike;
  final VoidCallback onOpenComments;
  final ValueChanged<String> onOpenHashtag;
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
  static const _overlayShadows = [
    Shadow(color: Color(0x99000000), blurRadius: 8),
  ];

  bool _isTogglingLike = false;
  bool _isCaptionExpanded = false;

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
        padding: const EdgeInsets.all(AppSpacing.md),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.end,
          children: [
            Expanded(child: _metadata(context)),
            const SizedBox(width: AppSpacing.md),
            _actionRail(),
          ],
        ),
      ),
    );
  }

  Widget _metadata(BuildContext context) {
    final theme = Theme.of(context).textTheme;
    final muted = theme.bodySmall?.copyWith(
      color: AppPalette.mutedForeground,
      shadows: _overlayShadows,
    );
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          widget.post.creator.displayName,
          style: theme.titleMedium?.copyWith(
            fontWeight: FontWeight.w700,
            shadows: _overlayShadows,
          ),
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
        ),
        const SizedBox(height: AppSpacing.xxs),
        Text(
          widget.post.creator.handle,
          style: muted,
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
        ),
        if (widget.post.caption.trim().isNotEmpty) ...[
          const SizedBox(height: AppSpacing.xs),
          _caption(context),
        ],
        const SizedBox(height: AppSpacing.xs),
        _songRow(muted),
      ],
    );
  }

  Widget _caption(BuildContext context) {
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: _toggleCaption,
      child: _isCaptionExpanded
          ? _expandedCaption(context)
          : _collapsedCaption(context),
    );
  }

  Widget _collapsedCaption(BuildContext context) {
    return CaptionText(
      caption: widget.post.caption,
      maxLines: 2,
      style: _captionStyle(context),
      onHashtagTap: widget.actions.onOpenHashtag,
    );
  }

  Widget _expandedCaption(BuildContext context) {
    return ConstrainedBox(
      constraints: BoxConstraints(
        maxHeight: MediaQuery.sizeOf(context).height * 0.35,
      ),
      child: SingleChildScrollView(
        child: CaptionText(
          caption: widget.post.caption,
          style: _captionStyle(context),
          onHashtagTap: widget.actions.onOpenHashtag,
        ),
      ),
    );
  }

  TextStyle? _captionStyle(BuildContext context) {
    return Theme.of(context)
        .textTheme
        .bodyMedium
        ?.copyWith(shadows: _overlayShadows);
  }

  void _toggleCaption() {
    setState(() => _isCaptionExpanded = !_isCaptionExpanded);
  }

  Widget _songRow(TextStyle? muted) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        const Icon(
          Icons.music_note,
          size: AppSize.feedSongIcon,
          color: AppPalette.mutedForeground,
          shadows: _overlayShadows,
        ),
        const SizedBox(width: AppSpacing.xxs),
        Flexible(
          child: Text(
            widget.post.songName,
            style: muted,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
        ),
      ],
    );
  }

  Widget _actionRail() {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        _profileButton(),
        const SizedBox(height: AppSpacing.lg),
        _likeButton(),
        const SizedBox(height: AppSpacing.md),
        _commentButton(),
      ],
    );
  }

  Widget _profileButton() {
    return Tooltip(
      message: 'Open profile',
      child: GestureDetector(
        onTap: widget.actions.onOpenProfile,
        child: DecoratedBox(
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            border: Border.all(color: AppPalette.foreground, width: 1.5),
          ),
          child: ProfileAvatar(
            initials: widget.post.creator.initials,
            avatarUrl: widget.post.creator.avatarUrl,
            radius: AppSize.feedRailAvatar,
          ),
        ),
      ),
    );
  }

  Widget _likeButton() {
    final isLiked = widget.post.viewerHasLiked;
    return _railButton(
      icon: isLiked ? Icons.favorite : Icons.favorite_border,
      iconColor: isLiked ? AppPalette.accentRed : AppPalette.foreground,
      count: widget.post.likeCount,
      tooltip: isLiked ? 'Unlike video' : 'Like video',
      onPressed: _isTogglingLike ? null : _toggleLike,
    );
  }

  Widget _commentButton() {
    return _railButton(
      icon: Icons.mode_comment,
      iconColor: AppPalette.foreground,
      count: widget.post.commentCount,
      tooltip: 'Open comments',
      onPressed: widget.actions.onOpenComments,
    );
  }

  Widget _railButton({
    required IconData icon,
    required Color iconColor,
    required int count,
    required String tooltip,
    required VoidCallback? onPressed,
  }) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        IconButton(
          onPressed: onPressed,
          tooltip: tooltip,
          iconSize: AppSize.feedRailIcon,
          icon: Icon(icon, color: iconColor, shadows: _overlayShadows),
        ),
        Text(
          '$count',
          style: Theme.of(context).textTheme.labelMedium?.copyWith(
                fontWeight: FontWeight.w600,
                shadows: _overlayShadows,
              ),
        ),
      ],
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
