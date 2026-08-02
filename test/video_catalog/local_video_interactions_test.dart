import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_video_interactions.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('keeps non-Nostr video interactions local and read-only', () async {
    final interactions = NostrVideoInteractions(
      FakeNostrEngagementPort(),
      FakeNostrCommentsPort(),
      RecordingFailureReporter(),
    );
    final post = samplePost();

    final liked = await interactions.toggleLike(post);
    final unliked = await interactions.toggleLike(liked);

    expect(liked.viewerHasLiked, isTrue);
    expect(liked.likeCount, post.likeCount + 1);
    expect(unliked.likeCount, post.likeCount);
    expect(await interactions.loadComments(post), isEmpty);
    expect(
      () => interactions.publishComment(post: post, content: 'No event'),
      throwsA(isA<AppFailure>()),
    );
  });
}
