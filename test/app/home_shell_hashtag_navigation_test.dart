import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/search_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/caption_text.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('activating a caption hashtag searches it on the search tab',
      (tester) async {
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(caption: 'Fresh #trend drop')],
    );
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: repository,
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    final caption = tester.widget<Text>(find.descendant(
      of: find.byType(CaptionText),
      matching: find.byType(Text),
    ));
    TapGestureRecognizer? recognizer;
    caption.textSpan!.visitChildren((span) {
      if (span is TextSpan && span.text == '#trend') {
        recognizer = span.recognizer as TapGestureRecognizer?;
        return false;
      }
      return true;
    });
    expect(recognizer, isNotNull);
    recognizer!.onTap!();
    await tester.pumpAndSettle();

    expect(find.byType(SearchScreen), findsOneWidget);
    expect(repository.searchQueries, ['#trend']);
  });
}
