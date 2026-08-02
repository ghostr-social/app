import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('ignores a comment load completion after disposal', () async {
    final pending = Completer<List<VideoComment>>();
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [],
      comments: FakeCommentsScenario(response: pending.future),
    );
    final cubit = CommentsCubit(repository, samplePost());

    final load = cubit.load();
    final completion = expectLater(load, completes);
    await cubit.close();
    pending.complete(const []);

    await completion;
  });
}
