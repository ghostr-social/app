import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/search_screen_harness.dart';

void main() {
  testWidgets('searches videos and opens the selected creator', (tester) async {
    String? openedProfileId;
    final creator = sampleCreator();
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(creator: creator)],
      feed: FakeFeedScenario(
        searchResults: [samplePost(creator: creator)],
      ),
    );
    await tester.pumpWidget(searchScreenHarness(
      repository,
      onOpenProfile: (id) => openedProfileId = id,
    ));

    await tester.enterText(find.byType(TextField), 'relay');
    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();
    await tester.tap(find.text(creator.displayName));

    expect(openedProfileId, creator.id);
  });
}
