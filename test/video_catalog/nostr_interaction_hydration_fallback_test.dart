import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_video_interactions.dart';

import '../support/fakes.dart';
import '../support/nostr_reference.dart';
import '../support/sample_data.dart';

void main() {
  test('keeps event counters when relay hydration fails', () async {
    final engagement = FakeNostrEngagementPort()
      ..loadFailure = const AppFailure('engagement unavailable');
    final comments = FakeNostrCommentsPort()
      ..loadFailure = const AppFailure('comments unavailable');
    final reporter = RecordingFailureReporter();
    final interactions = NostrVideoInteractions(
      engagement,
      comments,
      reporter,
    );
    final post = samplePost(
      nostrReference: nostrReference(
        eventId: testEventId,
        authorPublicKeyHex: testAuthorPublicKey,
      ),
    );

    final hydrated = await interactions.hydrate(post);

    expect(hydrated.likeCount, post.likeCount);
    expect(hydrated.commentCount, post.commentCount);
    expect(hydrated.viewerHasLiked, post.viewerHasLiked);
    expect(reporter.sources, [
      'NostrVideoInteractions.loadEngagement',
      'NostrVideoInteractions.loadCommentCount',
    ]);
  });
}
