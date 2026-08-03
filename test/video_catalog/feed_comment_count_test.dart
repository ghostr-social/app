import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/fake_video_catalog_repository.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('published comments update the visible feed count',
      (tester) async {
    final repository = FakeVideoCatalogRepository(forYouFeed: [samplePost()]);
    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();
    expect(find.text('9'), findsOneWidget);

    await tester.tap(find.byTooltip('Open comments'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'A new comment');
    await tester.pump();
    await tester.tap(find.byTooltip('Post comment'));
    await tester.pumpAndSettle();
    expect(find.text('A new comment'), findsOneWidget);
    await tester.binding.handlePopRoute();
    await tester.pumpAndSettle();

    expect(find.text('10'), findsOneWidget);
  });
}
