import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/comments/domain/video_comments_repository.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';

void main() {
  test('reply selection cannot reopen publication while one is pending',
      () async {
    final repository = _PendingCommentsRepository();
    final cubit = CommentsCubit(repository, samplePost());
    await cubit.load();

    final first = cubit.publish('First');
    cubit.selectReply(_comment());
    final second = cubit.publish('Second');
    final count = repository.publishCount;
    repository.complete();

    expect(await first, isTrue);
    expect(await second, isFalse);
    expect(count, 1);
    await cubit.close();
  });
}

class _PendingCommentsRepository implements VideoCommentsRepository {
  final _pending = Completer<VideoComment>();
  int publishCount = 0;

  @override
  Future<List<VideoComment>> loadComments(VideoPost post) async => const [];

  @override
  Future<VideoComment> publishComment({
    required VideoPost post,
    required String content,
    VideoComment? replyTo,
  }) {
    publishCount += 1;
    return _pending.future;
  }

  void complete() => _pending.complete(_comment());
}

VideoComment _comment() => VideoComment(
      identity: VideoCommentIdentity.parse(
        id: testEventId,
        authorPublicKeyHex: testCreatorPublicKey,
      ),
      text: VideoCommentText(authorLabel: 'Nora', content: 'First'),
      createdAt: DateTime.utc(2026, 8, 2),
    );
