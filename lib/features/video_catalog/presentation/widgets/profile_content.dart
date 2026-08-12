import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/profile_video_grid.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/profile_avatar.dart';

class ProfileContentActions {
  const ProfileContentActions({
    required this.onFollow,
    required this.onBlock,
    required this.onSignOut,
    this.onEdit,
    this.onOpenVideo,
  });

  final ValueChanged<ProfileId> onFollow;
  final ValueChanged<ProfileId> onBlock;
  final VoidCallback onSignOut;
  final VoidCallback? onEdit;
  final ValueChanged<VideoPost>? onOpenVideo;
}

class ProfileContent extends StatelessWidget {
  const ProfileContent({
    required this.details,
    required this.actions,
    required this.isUpdating,
    super.key,
  });

  final ProfileDetails details;
  final ProfileContentActions actions;
  final bool isUpdating;

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.all(AppSpacing.lg),
      children: [
        _ProfileHeader(details: details),
        const SizedBox(height: AppSpacing.lg),
        _ProfileActions(
          details: details,
          actions: actions,
          isUpdating: isUpdating,
        ),
        const SizedBox(height: AppSpacing.xl),
        Text('Videos', style: Theme.of(context).textTheme.titleLarge),
        const SizedBox(height: AppSpacing.sm),
        ProfileVideoGrid(
          posts: details.posts,
          onOpenVideo: actions.onOpenVideo,
        ),
      ],
    );
  }
}

class _ProfileHeader extends StatelessWidget {
  const _ProfileHeader({required this.details});

  final ProfileDetails details;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        ProfileAvatar(
          initials: details.profile.initials,
          avatarUrl: details.profile.avatarUrl,
          radius: AppSize.profileAvatar,
        ),
        const SizedBox(height: AppSpacing.md),
        Text(
          details.profile.displayName,
          style: Theme.of(context).textTheme.headlineMedium,
        ),
        const SizedBox(height: AppSpacing.xxs),
        Text(
          details.profile.handle,
          style: Theme.of(
            context,
          ).textTheme.bodyLarge?.copyWith(color: AppPalette.mutedForeground),
        ),
        const SizedBox(height: AppSpacing.lg),
        _ProfileMetrics(details: details),
      ],
    );
  }
}

class _ProfileMetrics extends StatelessWidget {
  const _ProfileMetrics({required this.details});

  final ProfileDetails details;

  @override
  Widget build(BuildContext context) {
    return Wrap(
      spacing: AppSpacing.sm,
      runSpacing: AppSpacing.sm,
      children: [
        _metric(context, details.posts.length, 'Posts'),
        _metric(context, details.totalLikes, 'Likes'),
        _metric(context, details.followingCount, 'Following'),
      ],
    );
  }

  Widget _metric(BuildContext context, int value, String label) {
    return Chip(
      backgroundColor: Theme.of(context).colorScheme.surface,
      label: Text('$value $label'),
    );
  }
}

class _ProfileActions extends StatelessWidget {
  const _ProfileActions({
    required this.details,
    required this.actions,
    required this.isUpdating,
  });

  final ProfileDetails details;
  final ProfileContentActions actions;
  final bool isUpdating;

  @override
  Widget build(BuildContext context) {
    if (details.isCurrentUser) {
      return _currentUserActions();
    }
    return Row(
      children: [
        Expanded(child: _followButton()),
        const SizedBox(width: AppSpacing.sm),
        Expanded(child: _blockButton()),
      ],
    );
  }

  Widget _currentUserActions() {
    final edit = actions.onEdit;
    if (edit == null) {
      return OutlinedButton(
        onPressed: actions.onSignOut,
        child: const Text('Sign out'),
      );
    }
    return Row(
      children: [
        Expanded(
          child: FilledButton.tonal(
            onPressed: edit,
            child: const Text('Edit profile'),
          ),
        ),
        const SizedBox(width: AppSpacing.sm),
        Expanded(
          child: OutlinedButton(
            onPressed: actions.onSignOut,
            child: const Text('Sign out'),
          ),
        ),
      ],
    );
  }

  Widget _followButton() {
    return FilledButton.tonal(
      onPressed: isUpdating ? null : () => actions.onFollow(details.profile.id),
      child: Text(details.isFollowing ? 'Following' : 'Follow'),
    );
  }

  Widget _blockButton() {
    return OutlinedButton(
      onPressed: isUpdating ? null : () => actions.onBlock(details.profile.id),
      child: Text(details.isBlocked ? 'Unblock' : 'Block'),
    );
  }
}
