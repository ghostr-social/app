import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';
import '../support/nostr_test_values.dart';

void main() {
  testWidgets('opens video comments and replies to a comment', (tester) async {
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost()],
      comments: FakeCommentsScenario(
        commentsByPost: {
          'post-1': [
            VideoComment(
              identity: VideoCommentIdentity.parse(
                id: testEventId,
                authorPublicKeyHex: testCreatorPublicKey,
              ),
              text: VideoCommentText(
                authorLabel: 'Alice',
                content: 'Love this clip',
              ),
              createdAt: DateTime(2026, 8, 2),
            ),
          ],
        },
      ),
    );
    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Open comments'));
    await tester.pumpAndSettle();
    expect(find.text('Love this clip'), findsOneWidget);

    await tester.tap(find.byTooltip('Reply to Alice'));
    await tester.enterText(find.byType(TextField), 'Exactly');
    await tester.tap(find.byTooltip('Post reply'));
    await tester.pumpAndSettle();

    expect(find.text('Exactly'), findsOneWidget);
  });
}
