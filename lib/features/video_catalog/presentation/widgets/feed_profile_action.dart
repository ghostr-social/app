import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';
import 'package:ghostr/shared/widgets/profile_avatar.dart';

class FeedProfileAction extends StatelessWidget {
  const FeedProfileAction({
    required this.profile,
    required this.onOpenProfile,
    this.onFollow,
    super.key,
  });

  final ProfileSummary profile;
  final VoidCallback onOpenProfile;
  final VoidCallback? onFollow;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: AppSize.feedProfileActionWidth,
      height: AppSize.feedProfileActionHeight,
      child: Stack(
        clipBehavior: Clip.none,
        alignment: Alignment.topCenter,
        children: [
          _avatar(),
          if (onFollow != null)
            Positioned(top: AppSize.feedFollowOffset, child: _followButton()),
        ],
      ),
    );
  }

  Widget _avatar() {
    return Tooltip(
      message: 'Open profile',
      child: GestureDetector(
        onTap: onOpenProfile,
        child: DecoratedBox(
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            border: Border.all(color: AppPalette.foreground, width: 1.5),
          ),
          child: ProfileAvatar(
            initials: profile.initials,
            avatarUrl: profile.avatarUrl,
            radius: AppSize.feedRailAvatar,
          ),
        ),
      ),
    );
  }

  // Sized to its visible circle so the avatar stays tappable beside the
  // badge it straddles.
  Widget _followButton() {
    final label = 'Follow ${profile.displayName}';
    return Semantics(
      button: true,
      label: label,
      onTap: onFollow,
      container: true,
      explicitChildNodes: true,
      child: ExcludeSemantics(
        child: IconButton(
          onPressed: onFollow,
          tooltip: label,
          style: IconButton.styleFrom(
            backgroundColor: AppPalette.accentRed,
            padding: EdgeInsets.zero,
            fixedSize: const Size.square(AppSize.feedFollowButton),
            minimumSize: const Size.square(AppSize.feedFollowButton),
            tapTargetSize: MaterialTapTargetSize.shrinkWrap,
          ),
          icon: const Icon(
            Icons.add,
            color: AppPalette.foreground,
            size: AppSize.feedFollowIcon,
          ),
        ),
      ),
    );
  }
}
