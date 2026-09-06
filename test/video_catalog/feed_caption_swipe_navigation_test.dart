import 'dart:ui' show SemanticsAction;

import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets(
    'swiping from a caption advances the video and keeps controls usable',
    (tester) async {
      final semantics = tester.ensureSemantics();
      try {
        final repository = FakeVideoCatalogRepository(
          forYouFeed: [
            samplePost(id: 'first', caption: 'First caption'),
            samplePost(id: 'second', caption: 'Second caption'),
          ],
        );
        await tester.pumpWidget(feedScreenHarness(repository));
        await tester.pumpAndSettle();

        await tester.drag(find.text('First caption'), const Offset(0, -400));
        await tester.pumpAndSettle();

        expect(find.text('Second caption').hitTestable(), findsOneWidget);
        expect(find.text('First caption'), findsNothing);
        final like = find.byTooltip('Like video').hitTestable();
        expect(like, findsOneWidget);
        expect(
          tester
              .getSemantics(like)
              .getSemanticsData()
              .hasAction(SemanticsAction.tap),
          isTrue,
        );
        await tester.tap(like);
        await tester.pumpAndSettle();
        expect(find.byTooltip('Unlike video').hitTestable(), findsOneWidget);
      } finally {
        semantics.dispose();
      }
    },
  );
}
