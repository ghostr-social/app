import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/search_screen_harness.dart';

void main() {
  testWidgets('searched creators open their profile from the creators row',
      (tester) async {
    ProfileId? openedProfileId;
    final creator = sampleCreator();
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [],
      feed: const FakeFeedScenario(searchResults: []),
    );
    repository.creatorResults.add(creator);
    await tester.pumpWidget(searchScreenHarness(
      repository,
      onOpenProfile: (id) => openedProfileId = id,
    ));

    await tester.enterText(find.byType(TextField), 'nora');
    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();

    expect(find.text('Creators'), findsOneWidget);
    await tester.tap(find.text(creator.displayName));

    expect(openedProfileId, creator.id);
  });
}
