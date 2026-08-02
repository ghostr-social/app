import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/search_screen_harness.dart';

void main() {
  testWidgets('shows the empty search state', (tester) async {
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [],
      feed: const FakeFeedScenario(searchResults: []),
    );
    await tester.pumpWidget(searchScreenHarness(repository));

    await tester.enterText(find.byType(TextField), 'none');
    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();

    expect(find.text('No matches found'), findsOneWidget);
  });
}
