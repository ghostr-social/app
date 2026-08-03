import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('removes a blocked creator after returning to the feed',
      (tester) async {
    final creator = sampleCreator(id: 'npub1blockedcreator');
    final catalog = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(creator: creator)],
      feed: FakeFeedScenario(profiles: {
        creator.id: sampleProfileDetails(profile: creator),
      }),
    );
    await tester.pumpWidget(buildTestApp(buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: catalog,
    )));
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Open profile'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Block'));
    await tester.pumpAndSettle();
    await tester.pageBack();
    await tester.pumpAndSettle();

    expect(find.text('Hunting for videos'), findsOneWidget);
    expect(find.text(creator.displayName), findsNothing);
  });
}
