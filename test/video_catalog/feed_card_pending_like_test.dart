import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';

import '../support/fake_media_ports.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('disables the like action while its write is pending',
      (tester) async {
    final result = Completer<void>();
    await tester.pumpWidget(MaterialApp(
      home: Scaffold(
        body: FeedCard(
          post: samplePost(),
          playbackPort: FakeVideoPlaybackPort(),
          isActive: true,
          actions: FeedCardActions(
            onOpenProfile: () {},
            onOpenHashtag: (_) {},
            onToggleLike: (_) => result.future,
            onOpenComments: () {},
            onBlockCreator: () {},
          ),
        ),
      ),
    ));

    await tester.tap(find.byTooltip('Like video'));
    await tester.pump();

    final button = tester.widget<IconButton>(find.byWidgetPredicate(
      (widget) => widget is IconButton && widget.tooltip == 'Like video',
    ));
    expect(button.onPressed, isNull);
    result.complete();
    await tester.pump();
  });
}
