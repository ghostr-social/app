import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('shows the empty comments state with a composer', (tester) async {
    final repository = FakeVideoCatalogRepository(forYouFeed: [samplePost()]);
    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('Open comments'));
    await tester.pumpAndSettle();

    expect(find.text('No comments yet'), findsOneWidget);
    expect(find.widgetWithText(TextField, 'Add a comment'), findsOneWidget);
  });
}
