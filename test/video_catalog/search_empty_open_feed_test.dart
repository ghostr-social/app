import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/search_screen_harness.dart';

void main() {
  testWidgets('an empty search still opens the query as a hunting feed',
      (tester) async {
    final opened = <String>[];
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [],
      feed: const FakeFeedScenario(searchResults: []),
    );
    await tester.pumpWidget(searchScreenHarness(
      repository,
      onOpenFeed: opened.add,
    ));

    await tester.enterText(find.byType(TextField), 'rare topic');
    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();

    expect(find.text('No matches found'), findsOneWidget);
    await tester.tap(find.text('Open in feed'));
    expect(opened, ['rare topic']);
  });
}
