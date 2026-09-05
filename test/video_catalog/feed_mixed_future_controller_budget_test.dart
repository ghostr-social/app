import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';

import '../support/mixed_future_playback_fixture.dart';

void main() {
  testWidgets(
    'mixed HLS and progressive readiness warms only the immediate next item',
    (tester) async {
      final fixture = MixedFuturePlaybackFixture();
      addTearDown(fixture.close);
      await fixture.prepare(tester, ['p4', 'p5', 'p6', 'p7', 'p8']);

      final cards = find.byType(FeedCard, skipOffstage: false);
      expect(cards, findsNWidgets(2));
      final ids = tester
          .widgetList<FeedCard>(cards)
          .map((card) => card.post.id.value);
      expect(ids, unorderedEquals(['p3', 'h0']));
    },
  );
}
