import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/search_screen_harness.dart';

void main() {
  testWidgets('creator-only results still offer opening the query as a feed',
      (tester) async {
    final opened = <String>[];
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [],
      feed: const FakeFeedScenario(searchResults: []),
    )..creatorResults.add(sampleCreator(displayName: 'Nora Relay'));
    await tester.pumpWidget(searchScreenHarness(
      repository,
      onOpenFeed: opened.add,
    ));

    await tester.enterText(find.byType(TextField), 'nora');
    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();

    expect(find.text('Nora Relay'), findsOneWidget);
    await tester.tap(find.byKey(const Key('open-in-feed')));
    expect(opened, ['nora']);
  });
}
