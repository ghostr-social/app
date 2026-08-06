import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/caption_text.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class FeedCardMetadata extends StatefulWidget {
  const FeedCardMetadata({
    required this.post,
    required this.onOpenHashtag,
    super.key,
  });

  final VideoPost post;
  final ValueChanged<String> onOpenHashtag;

  @override
  State<FeedCardMetadata> createState() => _FeedCardMetadataState();
}

class _FeedCardMetadataState extends State<FeedCardMetadata> {
  static const _shadows = [Shadow(color: Color(0x99000000), blurRadius: 8)];

  bool _isCaptionExpanded = false;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context).textTheme;
    final muted = theme.bodySmall?.copyWith(
      color: AppPalette.mutedForeground,
      shadows: _shadows,
    );
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _creatorName(theme),
        const SizedBox(height: AppSpacing.xxs),
        _creatorHandle(muted),
        if (widget.post.caption.trim().isNotEmpty) ...[
          const SizedBox(height: AppSpacing.xs),
          _caption(),
        ],
        const SizedBox(height: AppSpacing.xs),
        _songRow(muted),
      ],
    );
  }

  Widget _creatorName(TextTheme theme) {
    return Text(
      widget.post.creator.displayName,
      style: theme.titleMedium?.copyWith(
        fontWeight: FontWeight.w700,
        shadows: _shadows,
      ),
      maxLines: 1,
      overflow: TextOverflow.ellipsis,
    );
  }

  Widget _creatorHandle(TextStyle? style) {
    return Text(
      widget.post.creator.handle,
      style: style,
      maxLines: 1,
      overflow: TextOverflow.ellipsis,
    );
  }

  Widget _caption() {
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: _toggleCaption,
      child: _isCaptionExpanded ? _expandedCaption() : _collapsedCaption(),
    );
  }

  Widget _collapsedCaption() {
    return CaptionText(
      caption: widget.post.caption,
      maxLines: 2,
      style: _captionStyle(),
      onHashtagTap: widget.onOpenHashtag,
    );
  }

  Widget _expandedCaption() {
    return ConstrainedBox(
      constraints: BoxConstraints(
        maxHeight: MediaQuery.sizeOf(context).height * 0.35,
      ),
      child: SingleChildScrollView(
        child: CaptionText(
          caption: widget.post.caption,
          style: _captionStyle(),
          onHashtagTap: widget.onOpenHashtag,
        ),
      ),
    );
  }

  TextStyle? _captionStyle() {
    return Theme.of(context).textTheme.bodyMedium?.copyWith(shadows: _shadows);
  }

  void _toggleCaption() {
    setState(() => _isCaptionExpanded = !_isCaptionExpanded);
  }

  Widget _songRow(TextStyle? style) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        const Icon(
          Icons.music_note,
          size: AppSize.feedSongIcon,
          color: AppPalette.mutedForeground,
          shadows: _shadows,
        ),
        const SizedBox(width: AppSpacing.xxs),
        Flexible(
          child: Text(
            widget.post.songName,
            style: style,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
        ),
      ],
    );
  }
}
