import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card_actions.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_profile_action.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card_repost_action.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_origin.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

export 'feed_card_actions.dart';

class FeedCardActionRail extends StatefulWidget {
  const FeedCardActionRail({
    required this.post,
    required this.actions,
    super.key,
  });

  final VideoPost post;
  final FeedCardActions actions;

  @override
  State<FeedCardActionRail> createState() => _FeedCardActionRailState();
}

class _FeedCardActionRailState extends State<FeedCardActionRail> {
  bool _isTogglingLike = false;

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        FeedProfileAction(
          profile: widget.post.creator,
          onOpenProfile: widget.actions.navigation.onOpenProfile,
          onFollow: widget.actions.navigation.onFollowCreator == null
              ? null
              : _follow,
        ),
        const SizedBox(height: AppSpacing.lg),
        _likeButton(),
        const SizedBox(height: AppSpacing.md),
        _commentButton(),
        const SizedBox(height: AppSpacing.md),
        FeedCardRepostAction(
          post: widget.post,
          onToggle: widget.actions.engagement.onToggleRepost,
        ),
        const SizedBox(height: AppSpacing.md),
        _shareButton(),
      ],
    );
  }

  Future<void> _follow() {
    return widget.actions.navigation.onFollowCreator!(widget.post.creator);
  }

  Widget _likeButton() {
    final liked = widget.post.viewerHasLiked;
    return _countedButton(
      icon: Icon(
        liked ? Icons.favorite : Icons.favorite_border,
        color: liked ? AppPalette.accentRed : AppPalette.foreground,
        shadows: AppShadow.videoOverlay,
      ),
      count: widget.post.likeCount,
      tooltip: liked ? 'Unlike video' : 'Like video',
      onPressed: _isTogglingLike ? null : _toggleLike,
    );
  }

  Widget _commentButton() {
    return _countedButton(
      icon: const Icon(
        Icons.mode_comment,
        color: AppPalette.foreground,
        shadows: AppShadow.videoOverlay,
      ),
      count: widget.post.commentCount,
      tooltip: 'Open comments',
      onPressed: widget.actions.navigation.onOpenComments,
    );
  }

  Widget _shareButton() {
    return Builder(
      builder: (buttonContext) {
        final status = widget.actions.sharing.status;
        return IconButton(
          onPressed: status == FeedCardShareStatus.available
              ? () => _share(buttonContext)
              : null,
          tooltip: _shareTooltip(status),
          iconSize: AppSize.feedRailIcon,
          icon: status == FeedCardShareStatus.downloading
              ? const SizedBox.square(
                  dimension: AppSize.feedRailIcon,
                  child: CircularProgressIndicator(
                    strokeWidth: 2,
                    semanticsLabel: 'Downloading video to share',
                  ),
                )
              : const Icon(
                  Icons.share,
                  color: AppPalette.foreground,
                  shadows: AppShadow.videoOverlay,
                ),
        );
      },
    );
  }

  String _shareTooltip(FeedCardShareStatus status) {
    return switch (status) {
      FeedCardShareStatus.available => 'Share video',
      FeedCardShareStatus.unavailable => 'Sharing unavailable for this video',
      FeedCardShareStatus.downloading => 'Downloading video to share',
      FeedCardShareStatus.busy => 'Another video is being prepared',
    };
  }

  Future<void> _share(BuildContext context) {
    return widget.actions.sharing.onShare(widget.post, _shareOrigin(context));
  }

  VideoShareOrigin _shareOrigin(BuildContext context) {
    final box = context.findRenderObject();
    if (box is! RenderBox || !box.hasSize) return VideoShareOrigin.zero;
    final offset = box.localToGlobal(Offset.zero);
    return VideoShareOrigin(
      left: offset.dx,
      top: offset.dy,
      width: box.size.width,
      height: box.size.height,
    );
  }

  Widget _countedButton({
    required Icon icon,
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
          icon: icon,
        ),
        Text('$count', style: _countStyle()),
      ],
    );
  }

  TextStyle? _countStyle() {
    return Theme.of(context).textTheme.labelMedium?.copyWith(
      fontWeight: FontWeight.w600,
      shadows: AppShadow.videoOverlay,
    );
  }

  Future<void> _toggleLike() async {
    setState(() => _isTogglingLike = true);
    try {
      await widget.actions.engagement.onToggleLike(widget.post);
    } finally {
      if (mounted) setState(() => _isTogglingLike = false);
    }
  }
}
