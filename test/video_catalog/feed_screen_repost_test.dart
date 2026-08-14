import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/repost_samples.dart';

void main() {
  testWidgets('reposts a Nostr video through the feed screen', (tester) async {
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [repostablePost()],
    );
    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Repost video'));
    await tester.pumpAndSettle();

    expect(find.byTooltip('Undo repost'), findsOneWidget);
  });
}
