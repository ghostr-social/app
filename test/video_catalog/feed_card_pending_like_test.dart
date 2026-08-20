import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';

import '../support/fake_media_ports.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('disables the like action while its write is pending', (
    tester,
  ) async {
    final result = Completer<void>();
    final post = samplePost();
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: FeedCard(
            post: post,
            playback: FeedCardPlayback(
              port: FakeVideoPlaybackPort(),
              source: FeedCardPlaybackSource.direct(post.media),
              isActive: true,
            ),
            actions: FeedCardActions(
              navigation: FeedCardNavigationActions(
                onOpenProfile: () {},
                onOpenComments: () {},
                onOpenHashtag: (_) {},
              ),
              engagement: FeedCardEngagementActions(
                onToggleLike: (_) => result.future,
              ),
              moderation: FeedCardModerationActions(onBlockCreator: () {}),
              sharing: FeedCardSharingActions(onShare: (_, __) async {}),
            ),
          ),
        ),
      ),
    );

    await tester.tap(find.byTooltip('Like video'));
    await tester.pump();

    final button = tester.widget<IconButton>(
      find.byWidgetPredicate(
        (widget) => widget is IconButton && widget.tooltip == 'Like video',
      ),
    );
    expect(button.onPressed, isNull);
    result.complete();
    await tester.pump();
  });
}
