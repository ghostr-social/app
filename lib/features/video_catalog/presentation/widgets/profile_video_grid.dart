import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';

class ProfileVideoGrid extends StatelessWidget {
  const ProfileVideoGrid({required this.posts, super.key});

  final List<VideoPost> posts;

  @override
  Widget build(BuildContext context) {
    if (posts.isEmpty) {
      return const AsyncStatePanel(
        icon: Icons.video_collection_outlined,
        title: 'No videos published yet',
        message: 'This profile has not published anything in Ghostr yet.',
      );
    }
    return GridView.builder(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      itemCount: posts.length,
      gridDelegate: _gridDelegate,
      itemBuilder: (context, index) => _VideoTile(post: posts[index]),
    );
  }

  static const _gridDelegate = SliverGridDelegateWithFixedCrossAxisCount(
    crossAxisCount: 2,
    crossAxisSpacing: AppSpacing.sm,
    mainAxisSpacing: AppSpacing.sm,
    childAspectRatio: 0.95,
  );
}

class _VideoTile extends StatelessWidget {
  const _VideoTile({required this.post});

  final VideoPost post;

  @override
  Widget build(BuildContext context) {
    return DecoratedBox(
      decoration: _decoration(context),
      child: Padding(
        padding: const EdgeInsets.all(AppSpacing.md),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _caption(context),
            const Spacer(),
            _song(context),
          ],
        ),
      ),
    );
  }

  BoxDecoration _decoration(BuildContext context) {
    return BoxDecoration(
      color: Theme.of(context).colorScheme.surface,
      borderRadius: BorderRadius.circular(AppRadius.control),
    );
  }

  Widget _caption(BuildContext context) {
    return Text(
      post.caption,
      maxLines: 3,
      overflow: TextOverflow.ellipsis,
      style: Theme.of(context).textTheme.titleMedium,
    );
  }

  Widget _song(BuildContext context) {
    return Text(
      post.songName,
      maxLines: 2,
      overflow: TextOverflow.ellipsis,
      style: Theme.of(
        context,
      ).textTheme.bodySmall?.copyWith(color: AppPalette.mutedForeground),
    );
  }
}
