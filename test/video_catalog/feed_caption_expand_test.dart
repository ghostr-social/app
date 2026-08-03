import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/caption_text.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('expands the caption on tap and collapses it on a second tap',
      (tester) async {
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [
        samplePost(
          caption: 'First line of a long caption\n'
              'Second line with more detail\n'
              'Third line that keeps on going',
        ),
      ],
    );

    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();
    await tester.tap(find.byType(CaptionText));
    await tester.pumpAndSettle();

    expect(_captionText(tester).maxLines, isNull);
    expect(
      find.ancestor(
        of: find.byType(CaptionText),
        matching: find.byType(SingleChildScrollView),
      ),
      findsOneWidget,
    );

    await tester.tap(find.byType(CaptionText));
    await tester.pumpAndSettle();

    expect(_captionText(tester).maxLines, 2);
    expect(find.byType(SingleChildScrollView), findsNothing);
  });
}

Text _captionText(WidgetTester tester) {
  return tester.widget<Text>(find.descendant(
    of: find.byType(CaptionText),
    matching: find.byType(Text),
  ));
}
