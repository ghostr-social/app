import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_engagement.dart';

import '../support/sample_data.dart';

void main() {
  test('a rejected like hands back the original post and failure', () async {
    final post = samplePost();
    final engagement = FeedEngagement(
      _RejectingEngagement(const AppFailure('Relay rejected the like.')),
    );

    expect(engagement.optimistic(post).viewerHasLiked, isTrue);

    final result = await engagement.confirmLike(post);

    expect(result.post.viewerHasLiked, isFalse);
    expect(result.post.likeCount, post.likeCount);
    expect(
      (result.failure?.cause as AppFailure).message,
      'Relay rejected the like.',
    );
  });

  test('an unexpected like error retains its diagnostic context', () async {
    final engagement = FeedEngagement(_RejectingEngagement(StateError('boom')));

    final result = await engagement.confirmLike(samplePost());

    expect(result.failure?.cause, isA<StateError>());
    expect(result.failure?.stackTrace, isNot(StackTrace.empty));
  });
}

class _RejectingEngagement implements VideoEngagementRepository {
  _RejectingEngagement(this.error);

  final Object error;

  @override
  Future<VideoPost> toggleLike(VideoPost post) async => throw error;
}
