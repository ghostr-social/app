import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card_action_rail.dart';
import 'package:ghostr/shared/theme/app_tokens.dart';

import '../support/repost_samples.dart';

void main() {
  testWidgets('repost action exposes pending and toggled semantics', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    final gate = Completer<void>();
    var calls = 0;
    var post = repostablePost();
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
                    calls += 1;
                    setState(() => post = post.withRepost(true));
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

    expect(
      tester.getSemantics(find.byTooltip('Repost video')),
      isSemantics(
        tooltip: 'Repost video',
        hasSelectedState: true,
        isSelected: false,
        isButton: true,
        hasTapAction: true,
      ),
    );

    await tester.tap(find.byTooltip('Repost video'));
    await tester.pump();
    expect(find.bySemanticsLabel('Reposting video'), findsOneWidget);
    final pending = tester.widget<IconButton>(
      find.byWidgetPredicate(
        (widget) => widget is IconButton && widget.tooltip == 'Repost video',
      ),
    );
    expect(pending.onPressed, isNull);
    expect(calls, 1);

    gate.complete();
    await tester.pumpAndSettle();
    expect(
      tester.getSemantics(find.byTooltip('Undo repost')),
      isSemantics(
        tooltip: 'Undo repost',
        hasSelectedState: true,
        isSelected: true,
        isButton: true,
        hasTapAction: true,
      ),
    );
    expect(
      tester.widget<Icon>(find.byIcon(Icons.repeat)).color,
      AppPalette.accentBlue,
    );
    semantics.dispose();
  });
}
