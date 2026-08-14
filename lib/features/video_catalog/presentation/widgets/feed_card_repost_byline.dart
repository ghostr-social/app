import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_attribution.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class FeedCardRepostByline extends StatelessWidget {
  const FeedCardRepostByline({required this.repost, super.key});

  final VideoRepostAttribution repost;

  @override
  Widget build(BuildContext context) {
    final style = Theme.of(context).textTheme.labelMedium?.copyWith(
      color: AppPalette.mutedForeground,
      shadows: AppShadow.videoOverlay,
    );
    return Semantics(
      label: '${repost.reposter.displayName} reposted this video',
      excludeSemantics: true,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(
            Icons.repeat,
            size: AppSize.feedSongIcon,
            color: AppPalette.mutedForeground,
          ),
          const SizedBox(width: AppSpacing.xxs),
          Flexible(
            child: Text(
              '${repost.reposter.displayName} reposted',
              style: style,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ],
      ),
    );
  }
}
