import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('cancelling a caption drag restores the committed video', (
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
    final gesture = await tester.startGesture(
      tester.getCenter(find.text('First caption')),
    );
    await gesture.moveBy(const Offset(0, -120));
    await tester.pump();
    final controller = tester
        .widget<PageView>(find.byType(PageView))
        .controller!;
    expect(
      controller.page,
      greaterThan(0),
      reason: 'the caption drag moves media',
    );
    expect(find.text('First caption').hitTestable(), findsOneWidget);

    await gesture.cancel();
    await tester.pumpAndSettle();

    expect(controller.page, 0);
    expect(find.text('First caption').hitTestable(), findsOneWidget);
    expect(find.text('Second caption'), findsNothing);
    expect(find.byTooltip('Like video').hitTestable(), findsOneWidget);
  });
}
