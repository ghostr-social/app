import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/profile_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('renders a creator profile with follow action', (tester) async {
    final creator = sampleCreator();
    final catalog = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(creator: creator)],
      feed: FakeFeedScenario(profiles: {
        creator.id: sampleProfileDetails(
          profile: creator,
          posts: [samplePost(creator: creator)],
        ),
      }),
    );
    await tester.pumpWidget(profileScreenHarness(
      profile: catalog,
      viewer: sampleSession().profile,
      profileId: creator.id,
    ));
    await tester.pumpAndSettle();

    expect(find.text(creator.displayName), findsOneWidget);
    expect(find.text('Follow'), findsOneWidget);
  });
}
