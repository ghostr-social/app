import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card_action_rail.dart';

import '../support/repost_samples.dart';

void main() {
  testWidgets('unrepost keeps its removal semantics after optimism', (
    tester,
  ) async {
    final gate = Completer<void>();
    var post = repostablePost().withRepost(true);
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: StatefulBuilder(
            builder: (context, setState) => FeedCardActionRail(
              post: post,
              actions: FeedCardActions(
                navigation: FeedCardNavigationActions(
                  onOpenProfile: () {},
                  onOpenComments: () {},
                  onOpenHashtag: (_) {},
                ),
                engagement: FeedCardEngagementActions(
                  onToggleLike: (_) async {},
                  onToggleRepost: (_) async {
                    setState(() => post = post.withRepost(false));
                    await gate.future;
                  },
                ),
                moderation: FeedCardModerationActions(onBlockCreator: () {}),
                sharing: FeedCardSharingActions(onShare: (_, __) async {}),
              ),
            ),
          ),
        ),
      ),
    );

    await tester.tap(find.byTooltip('Undo repost'));
    await tester.pump();

    expect(find.bySemanticsLabel('Removing repost'), findsOneWidget);
    expect(find.byTooltip('Undo repost'), findsOneWidget);

    gate.complete();
    await tester.pumpAndSettle();
  });
}
