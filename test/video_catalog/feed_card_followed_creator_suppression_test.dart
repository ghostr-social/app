import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('does not offer to follow an already-followed feed creator', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    final viewer = sampleSession().profile;
    final creator = sampleCreator();
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(creator: creator)],
    );
    repository.followedProfiles.add(creator.id);

    await tester.pumpWidget(
      feedScreenHarness(
        repository,
        options: FeedScreenHarnessOptions(viewerId: viewer.id),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.byTooltip('Open profile'), findsOneWidget);
    expect(
      find.bySemanticsLabel('Follow ${creator.displayName}'),
      findsNothing,
    );
    semantics.dispose();
  });
}
