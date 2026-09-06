import 'dart:ui' show SemanticsAction;

import 'package:flutter_test/flutter_test.dart';

import '../support/fake_video_catalog_repository.dart';
import '../support/fake_video_catalog_scenarios.dart';
import '../support/feed_preparation_fixture.dart';
import '../support/feed_preview_gestures.dart';

void main() {
  testWidgets('switching feeds resets the preview direction for the new feed', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture();
    addTearDown(fixture.updates.close);
    final semantics = tester.ensureSemantics();
    try {
      await fixture.pump(
        tester,
        feed: FakeVideoCatalogRepository(
          forYouFeed: fixture.posts,
          feed: FakeFeedScenario(followingFeed: fixture.posts),
        ),
      );
      fixture.publish(1, 'p0', 'p1');
      await fixture.settle(tester);
      await fixture.swipe(tester);
      fixture.publishWindow(2, 'p1', ['p0', 'p2']);
      await fixture.settle(tester);
      final back = await tester.dragPreview(0.23);
      await back.up();
      await fixture.settle(tester);
      expect(fixture.platform.playerCount, 1);

      final following = find.text('Following').hitTestable();
      expect(
        tester
            .getSemantics(following)
            .getSemanticsData()
            .hasAction(SemanticsAction.tap),
        isTrue,
      );
      await tester.tap(following);
      await fixture.settle(tester);
      fixture.publish(3, 'p0', 'p1');
      await fixture.settle(tester);

      expect(find.text('Caption p0').hitTestable(), findsOneWidget);
      expect(fixture.platform.playerCount, 2);
      expect(fixture.platform.peakPlayerCount, lessThanOrEqualTo(2));
      expect(fixture.platform.audibleOverlap, isFalse);
      expect(find.byTooltip('Like video').hitTestable(), findsOneWidget);
    } finally {
      semantics.dispose();
    }
  });
}
