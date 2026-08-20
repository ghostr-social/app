import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';

import '../support/fake_media_ports.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('renders a replacement post when the card state is reused', (
    tester,
  ) async {
    final actions = FeedCardActions(
      navigation: FeedCardNavigationActions(
        onOpenProfile: () {},
        onOpenComments: () {},
        onOpenHashtag: (_) {},
      ),
      engagement: FeedCardEngagementActions(onToggleLike: (post) async {}),
      moderation: FeedCardModerationActions(onBlockCreator: () {}),
      sharing: FeedCardSharingActions(onShare: (_, __) async {}),
    );

    await tester.pumpWidget(
      _app(
        post: samplePost(id: 'first', caption: 'First caption'),
        actions: actions,
      ),
    );
    await tester.pumpWidget(
      _app(
        post: samplePost(id: 'second', caption: 'Second caption'),
        actions: actions,
      ),
    );

    expect(find.text('Second caption'), findsOneWidget);
    expect(find.text('First caption'), findsNothing);
  });
}

Widget _app({required VideoPost post, required FeedCardActions actions}) {
  return MaterialApp(
    home: Scaffold(
      body: FeedCard(
        post: post,
        playback: FeedCardPlayback(
          port: FakeVideoPlaybackPort(),
          source: FeedCardPlaybackSource.direct(post.media),
          isActive: true,
        ),
        actions: actions,
      ),
    ),
  );
}
