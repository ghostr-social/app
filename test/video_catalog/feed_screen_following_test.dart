import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('switches from For You to Following', (tester) async {
    final creator = sampleCreator();

    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(caption: 'For you only', creator: creator)],
      feed: FakeFeedScenario(followingFeed: [
        samplePost(caption: 'Following only', creator: creator),
      ]),
    );
    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();

    expect(find.text('For you only'), findsOneWidget);
    await tester.tap(find.text('Following'));
    await tester.pumpAndSettle();

    expect(find.text('Following only'), findsOneWidget);
  });
}
