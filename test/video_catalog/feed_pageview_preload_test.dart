import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('the feed keeps neighbouring pages mounted for instant swipes',
      (tester) async {
    final repository = FakeVideoCatalogRepository(forYouFeed: [
      samplePost(id: 'post-0'),
      samplePost(id: 'post-1'),
    ]);
    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();

    final pageView = tester.widget<PageView>(find.byType(PageView));

    expect(pageView.allowImplicitScrolling, isTrue);
  });
}
