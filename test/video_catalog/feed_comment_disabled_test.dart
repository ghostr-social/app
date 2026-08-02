import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('disables comment posting until text is entered', (tester) async {
    final repository = FakeVideoCatalogRepository(forYouFeed: [samplePost()]);
    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('Open comments'));
    await tester.pumpAndSettle();

    final sendButton = find.ancestor(
      of: find.byTooltip('Post comment'),
      matching: find.byType(IconButton),
    );
    IconButton button = tester.widget(sendButton);
    expect(button.onPressed, isNull);

    await tester.enterText(find.byType(TextField), 'A comment');
    await tester.pump();

    button = tester.widget(sendButton);
    expect(button.onPressed, isNotNull);
  });
}
