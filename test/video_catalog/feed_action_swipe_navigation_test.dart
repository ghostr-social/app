import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('swiping from Like navigates both ways without liking a video', (
    tester,
  ) async {
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [
        samplePost(id: 'first', caption: 'First caption'),
        samplePost(id: 'second', caption: 'Second caption'),
      ],
    );
    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();

    await tester.drag(find.byTooltip('Like video'), const Offset(0, -400));
    await tester.pumpAndSettle();
    expect(find.text('Second caption').hitTestable(), findsOneWidget);
    expect(find.byTooltip('Unlike video'), findsNothing);

    await tester.drag(find.byTooltip('Like video'), const Offset(0, 400));
    await tester.pumpAndSettle();
    expect(find.text('First caption').hitTestable(), findsOneWidget);
    expect(find.byTooltip('Unlike video'), findsNothing);
    await tester.tap(find.byTooltip('Like video'));
    await tester.pumpAndSettle();
    expect(find.byTooltip('Unlike video').hitTestable(), findsOneWidget);
  });
}
