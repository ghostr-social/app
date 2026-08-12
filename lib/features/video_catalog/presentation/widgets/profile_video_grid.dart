import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';

class ProfileVideoGrid extends StatelessWidget {
  const ProfileVideoGrid({required this.posts, this.onOpenVideo, super.key});

  final List<VideoPost> posts;
  final ValueChanged<VideoPost>? onOpenVideo;

  /// The tap target of one video on the grid.
  static Key tileKey(VideoPostId id) => ValueKey('profile-video-${id.value}');

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
      itemBuilder: (context, index) =>
          _VideoTile(post: posts[index], onOpen: onOpenVideo),
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
  const _VideoTile({required this.post, required this.onOpen});

  final VideoPost post;
  final ValueChanged<VideoPost>? onOpen;

  @override
  Widget build(BuildContext context) {
    return MergeSemantics(
      key: ProfileVideoGrid.tileKey(post.id),
      child: Semantics(button: onOpen != null, child: _surface(context)),
    );
  }

  Widget _surface(BuildContext context) {
    final open = onOpen;
    final shape = BorderRadius.circular(AppRadius.control);
    return Material(
      color: Theme.of(context).colorScheme.surface,
      borderRadius: shape,
      child: InkWell(
        borderRadius: shape,
        onTap: open == null ? null : () => open(post),
        child: _body(context),
      ),
    );
  }

  Widget _body(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(AppSpacing.md),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [_caption(context), const Spacer(), _song(context)],
      ),
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
