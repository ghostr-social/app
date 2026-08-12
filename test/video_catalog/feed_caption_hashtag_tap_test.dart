import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/caption_text.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('opens a hashtag with its full text when its span is activated', (
    tester,
  ) async {
    final openedHashtags = <String>[];
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(caption: 'Relay tricks for #nostrdev fans')],
    );

    await tester.pumpWidget(
      feedScreenHarness(
        repository,
        options: FeedScreenHarnessOptions(onOpenHashtag: openedHashtags.add),
      ),
    );
    await tester.pumpAndSettle();

    final caption = tester.widget<Text>(
      find.descendant(
        of: find.byType(CaptionText),
        matching: find.byType(Text),
      ),
    );
    TapGestureRecognizer? recognizer;
    caption.textSpan!.visitChildren((span) {
      if (span is TextSpan && span.text == '#nostrdev') {
        recognizer = span.recognizer as TapGestureRecognizer?;
        return false;
      }
      return true;
    });
    expect(recognizer, isNotNull);
    recognizer!.onTap!();
    await tester.pump();

    expect(openedHashtags, ['#nostrdev']);
  });
}
