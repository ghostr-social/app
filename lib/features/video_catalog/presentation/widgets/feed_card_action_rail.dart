import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_profile_action.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_origin.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

enum FeedCardShareStatus { available, unavailable, downloading, busy }

class FeedCardActions {
  const FeedCardActions({
    required this.onOpenProfile,
    this.onFollowCreator,
    required this.onToggleLike,
    required this.onOpenComments,
    required this.onOpenHashtag,
    required this.onBlockCreator,
    required this.onShare,
    this.shareStatus = FeedCardShareStatus.available,
  });

  final VoidCallback onOpenProfile;
  final Future<void> Function(ProfileSummary creator)? onFollowCreator;
  final Future<void> Function(VideoPost post) onToggleLike;
  final VoidCallback onOpenComments;
  final ValueChanged<String> onOpenHashtag;
  final VoidCallback onBlockCreator;
  final Future<void> Function(VideoPost post, VideoShareOrigin origin) onShare;
  final FeedCardShareStatus shareStatus;
}

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
  static const _shadows = [Shadow(color: Color(0x99000000), blurRadius: 8)];

  bool _isTogglingLike = false;

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        FeedProfileAction(
          profile: widget.post.creator,
          onOpenProfile: widget.actions.onOpenProfile,
          onFollow: widget.actions.onFollowCreator == null ? null : _follow,
        ),
        const SizedBox(height: AppSpacing.lg),
        _likeButton(),
        const SizedBox(height: AppSpacing.md),
        _commentButton(),
        const SizedBox(height: AppSpacing.md),
        _shareButton(),
      ],
    );
  }

  Future<void> _follow() {
    return widget.actions.onFollowCreator!(widget.post.creator);
  }

  Widget _likeButton() {
    final liked = widget.post.viewerHasLiked;
    return _countedButton(
      icon: liked ? Icons.favorite : Icons.favorite_border,
      color: liked ? AppPalette.accentRed : AppPalette.foreground,
      count: widget.post.likeCount,
      tooltip: liked ? 'Unlike video' : 'Like video',
      onPressed: _isTogglingLike ? null : _toggleLike,
    );
  }

  Widget _commentButton() {
    return _countedButton(
      icon: Icons.mode_comment,
      color: AppPalette.foreground,
      count: widget.post.commentCount,
      tooltip: 'Open comments',
      onPressed: widget.actions.onOpenComments,
    );
  }

  Widget _shareButton() {
    return Builder(
      builder: (buttonContext) {
        final status = widget.actions.shareStatus;
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
                  shadows: _shadows,
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
    return widget.actions.onShare(widget.post, _shareOrigin(context));
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
    required IconData icon,
    required Color color,
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
          icon: Icon(icon, color: color, shadows: _shadows),
        ),
        Text('$count', style: _countStyle()),
      ],
    );
  }

  TextStyle? _countStyle() {
    return Theme.of(context).textTheme.labelMedium?.copyWith(
      fontWeight: FontWeight.w600,
      shadows: _shadows,
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
