import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('likes and unlikes a Nostr video from the feed', (tester) async {
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost()],
    );

    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('Like video'));
    await tester.pumpAndSettle();

    expect(find.text('43'), findsOneWidget);
    expect(find.byTooltip('Unlike video'), findsOneWidget);
  });
}
