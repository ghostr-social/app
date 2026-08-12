import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/fake_video_catalog_repository.dart';
import '../support/fake_video_sharing.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('disables another card while a video download is pending', (
    tester,
  ) async {
    final pending = Completer<void>();
    final sharing = FakeVideoShareWorkflow(pending: pending);
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [
        samplePost(id: 'first'),
        samplePost(id: 'second'),
      ],
    );
    await tester.pumpWidget(
      feedScreenHarness(
        repository,
        options: FeedScreenHarnessOptions(shareWorkflow: sharing),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Share video'));
    await tester.pump();
    await tester.fling(find.byType(PageView), const Offset(0, -500), 1000);
    await tester.pump(const Duration(seconds: 1));

    final button = tester.widget<IconButton>(
      find.byWidgetPredicate(
        (widget) =>
            widget is IconButton &&
            widget.tooltip == 'Another video is being prepared',
      ),
    );
    expect(button.onPressed, isNull);
    pending.complete();
    await tester.pump();
  });
}
