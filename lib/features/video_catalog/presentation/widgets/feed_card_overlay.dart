import 'package:flutter/material.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card_action_rail.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card_metadata.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

class FeedCardOverlay extends StatelessWidget {
  const FeedCardOverlay({required this.post, required this.actions, super.key});

  final VideoPost post;
  final FeedCardActions actions;

  @override
  Widget build(BuildContext context) => Stack(
    fit: StackFit.expand,
    children: [
      const IgnorePointer(child: _FeedScrim()),
      _content(),
    ],
  );

  Widget _content() {
    return SafeArea(
      child: LayoutBuilder(
        builder: (_, constraints) => _overlay(constraints.maxHeight),
      ),
    );
  }

  Widget _overlay(double height) {
    return Padding(
      padding: const EdgeInsets.all(AppSpacing.md),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          Expanded(
            child: FeedCardMetadata(
              post: post,
              onOpenHashtag: actions.navigation.onOpenHashtag,
            ),
          ),
          const SizedBox(width: AppSpacing.md),
          _rail(height - AppSpacing.md * 2),
        ],
      ),
    );
  }

  Widget _rail(double height) {
    final rail = FeedCardActionRail(post: post, actions: actions);
    if (height >= AppSize.feedRailMinHeight) return rail;
    return SizedBox(
      height: height,
      child: SingleChildScrollView(reverse: true, child: rail),
    );
  }
}

class _FeedScrim extends StatelessWidget {
  const _FeedScrim();

  @override
  Widget build(BuildContext context) {
    return const DecoratedBox(
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [AppPalette.videoScrimTop, AppPalette.videoScrimBottom],
        ),
      ),
    );
  }
}
