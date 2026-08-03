import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/fake_video_catalog_repository.dart';
import '../support/fake_video_catalog_scenarios.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('a comment finishing after dismissal still updates the feed',
      (tester) async {
    final barrier = Completer<void>();
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost()],
      comments: FakeCommentsScenario(publishBarrier: barrier.future),
    );
    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Open comments'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'Finishes later');
    await tester.pump();
    await tester.tap(find.byTooltip('Post comment'));
    await tester.pump();
    await tester.binding.handlePopRoute();
    await tester.pumpAndSettle();
    expect(find.text('9'), findsOneWidget);

    barrier.complete();
    await tester.pumpAndSettle();

    expect(find.text('10'), findsOneWidget);
  });
}
