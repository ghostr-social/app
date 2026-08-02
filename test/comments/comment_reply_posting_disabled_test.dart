import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/comments/presentation/video_comment_tile.dart';

import '../support/nostr_test_values.dart';

void main() {
  testWidgets('a pending comment disables its reply action', (tester) async {
    final comment = VideoComment(
      identity: VideoCommentIdentity.parse(
        id: testEventId,
        authorPublicKeyHex: testCreatorPublicKey,
      ),
      text: VideoCommentText(authorLabel: 'Nora', content: 'Relay reply'),
      createdAt: DateTime.utc(2026, 8, 2),
    );

    await tester.pumpWidget(MaterialApp(
      home: Scaffold(
        body: VideoCommentTile(comment: comment, onReply: null),
      ),
    ));

    expect(
        tester.widget<IconButton>(find.byType(IconButton)).onPressed, isNull);
  });
}
