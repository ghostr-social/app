import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('shows a comment publishing failure from the comments sheet',
      (tester) async {
    final repository = _RejectedCommentRepository();
    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('Open comments'));
    await tester.pumpAndSettle();

    await tester.enterText(find.byType(TextField), 'Relay comment');
    await tester.pump();
    await tester.tap(find.byTooltip('Post comment'));
    await tester.pump(const Duration(seconds: 1));

    expect(repository.publishCount, 1);
    expect(find.byType(SnackBar), findsOneWidget);
    expect(find.text('Relay rejected the comment.'), findsOneWidget);
  });
}

class _RejectedCommentRepository extends FakeVideoCatalogRepository {
  _RejectedCommentRepository() : super(forYouFeed: [samplePost()]);

  int publishCount = 0;

  @override
  Future<VideoComment> publishComment({
    required VideoPost post,
    required String content,
    VideoComment? replyTo,
  }) {
    publishCount += 1;
    throw const AppFailure('Relay rejected the comment.');
  }
}
