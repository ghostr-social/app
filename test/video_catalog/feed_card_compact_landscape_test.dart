import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';

import '../support/fake_media_ports.dart';
import '../support/repost_samples.dart';

void main() {
  testWidgets('action rail stays usable in compact landscape', (tester) async {
    tester.view.physicalSize = const Size(640, 280);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: FeedCard(
            post: repostablePost(),
            playback: FeedCardPlayback(
              port: FakeVideoPlaybackPort(),
              isActive: true,
            ),
            actions: FeedCardActions(
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
            ),
          ),
        ),
      ),
    );
    await tester.pump();

    expect(tester.takeException(), isNull);
    expect(
      find.byWidgetPredicate(
        (widget) =>
            widget is SingleChildScrollView &&
            widget.scrollDirection == Axis.vertical,
      ),
      findsOneWidget,
    );
  });
}
