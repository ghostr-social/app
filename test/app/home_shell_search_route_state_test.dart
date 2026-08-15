import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('search query survives while the opened video stays excluded', (
    tester,
  ) async {
    final post = samplePost(id: 'search-post');
    final fresh = samplePost(
      id: 'fresh-search-post',
      creator: sampleCreator(id: 'fresh', displayName: 'Fresh Relay'),
    );
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: [samplePost(id: 'home-post')],
        feed: FakeFeedScenario(
          searchResults: [post, fresh],
          profiles: {post.creator.id: sampleProfileDetails()},
        ),
      ),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Search').last);
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'relay query');
    await tester.tap(find.widgetWithText(FilledButton, 'Search'));
    await tester.pumpAndSettle();
    await tester.tap(find.text(post.creator.displayName));
    await tester.pumpAndSettle();
    await tester.pageBack();
    await tester.pumpAndSettle();

    expect(find.text('relay query'), findsOneWidget);
    expect(find.text(post.creator.displayName), findsNothing);
    expect(find.text(fresh.creator.displayName), findsOneWidget);
  });
}
