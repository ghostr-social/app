import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('announces the comments loading state', (tester) async {
    final completer = Completer<List<VideoComment>>();
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost()],
      comments: FakeCommentsScenario(response: completer.future),
    );
    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('Open comments'));
    await tester.pump();

    expect(find.bySemanticsLabel('Loading comments'), findsOneWidget);

    completer.complete([]);
    await tester.pumpAndSettle();
    expect(find.text('No comments yet'), findsOneWidget);
  });
}
