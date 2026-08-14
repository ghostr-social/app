import 'package:flutter_test/flutter_test.dart';

import '../support/fake_video_catalog_repository.dart';
import '../support/fake_video_catalog_scenarios.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('viewer can open the Following feed from home', (tester) async {
    final semantics = tester.ensureSemantics();
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(id: 'for-you', caption: 'For You clip')],
      feed: FakeFeedScenario(
        followingFeed: [samplePost(id: 'following', caption: 'Following clip')],
      ),
    );

    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();
    expect(
      tester.getSemantics(find.bySemanticsLabel('For You')),
      isSemantics(
        label: 'For You',
        hasSelectedState: true,
        isSelected: true,
        isButton: true,
        hasTapAction: true,
      ),
    );
    await tester.tap(find.text('Following'));
    await tester.pumpAndSettle();

    expect(find.text('Following clip'), findsOneWidget);
    expect(find.text('For You clip'), findsNothing);
    expect(
      tester.getSemantics(find.bySemanticsLabel('Following')),
      isSemantics(
        label: 'Following',
        hasSelectedState: true,
        isSelected: true,
        isButton: true,
        hasTapAction: true,
      ),
    );
    semantics.dispose();
  });
}
