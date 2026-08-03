import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/search_screen_harness.dart';

void main() {
  testWidgets('search results open as a swipeable feed of the query',
      (tester) async {
    final opened = <String>[];
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(caption: 'Relay banger')],
    );
    await tester.pumpWidget(searchScreenHarness(
      repository,
      onOpenFeed: opened.add,
    ));

    await tester.enterText(find.byType(TextField), 'relay');
    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const Key('open-in-feed')));
    expect(opened, ['relay']);

    await tester.tap(find.text('Relay banger'));
    expect(opened, ['relay', 'relay']);
  });
}
