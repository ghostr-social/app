import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card_action_rail.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card_metadata.dart';
import 'package:ghostr/shared/widgets/profile_avatar.dart';

import '../support/repost_samples.dart';

void main() {
  testWidgets('repost card keeps the original creator avatar in its rail', (
    tester,
  ) async {
    final post = repostedPost();
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Column(
            children: [
              FeedCardActionRail(post: post, actions: actions()),
              FeedCardMetadata(post: post, onOpenHashtag: (_) {}),
            ],
          ),
        ),
      ),
    );

    final avatar = find.byType(ProfileAvatar);
    expect(
      find.descendant(of: avatar, matching: find.text('NR')),
      findsOneWidget,
    );
    expect(
      find.descendant(of: avatar, matching: find.text('BR')),
      findsNothing,
    );
    expect(find.text('Bob Relay reposted'), findsOneWidget);
  });
}

FeedCardActions actions() {
  return FeedCardActions(
    navigation: FeedCardNavigationActions(
      onOpenProfile: () {},
      onOpenComments: () {},
      onOpenHashtag: (_) {},
    ),
    engagement: FeedCardEngagementActions(
      onToggleLike: (_) async {},
      onToggleRepost: (_) async {},
    ),
    moderation: FeedCardModerationActions(onBlockCreator: () {}),
    sharing: FeedCardSharingActions(onShare: (_, __) async {}),
  );
}
