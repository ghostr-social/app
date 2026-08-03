import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/caption_text.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('renders a multi-line caption collapsed to two ellipsized lines',
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

    final caption = tester.widget<Text>(find.descendant(
      of: find.byType(CaptionText),
      matching: find.byType(Text),
    ));
    expect(caption.maxLines, 2);
    expect(caption.overflow, TextOverflow.ellipsis);
  });
}
