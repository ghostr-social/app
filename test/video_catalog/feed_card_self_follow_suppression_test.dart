import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('does not offer to follow the signed-in viewer in the feed', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    final viewer = sampleSession().profile;
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(creator: viewer)],
    );

    await tester.pumpWidget(
      feedScreenHarness(
        repository,
        options: FeedScreenHarnessOptions(viewerId: viewer.id),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.byTooltip('Open profile'), findsOneWidget);
    expect(find.bySemanticsLabel('Follow ${viewer.displayName}'), findsNothing);
    semantics.dispose();
  });
}
