import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/profile_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('blocks and unblocks a creator from their profile',
      (tester) async {
    final creator = sampleCreator(id: 'npub1creator');
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(creator: creator)],
      feed: FakeFeedScenario(profiles: {
        creator.id: sampleProfileDetails(
            profile: creator, posts: [samplePost(creator: creator)]),
      }),
    );

    await tester.pumpWidget(profileScreenHarness(
      profile: repository,
      viewer: sampleSession().profile,
      profileId: creator.id,
    ));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Block'));
    await tester.pumpAndSettle();

    expect(find.text('Unblock'), findsOneWidget);
  });
}
