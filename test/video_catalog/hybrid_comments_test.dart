import 'package:flutter_test/flutter_test.dart';
import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/nostr_reference.dart';
import '../support/sample_data.dart';

void main() {
  test('delegates a Nostr video comment and reply through the catalog',
      () async {
    final reference = nostrReference(
      eventId: testEventId,
    );
    final comments = FakeNostrCommentsPort();
    final harness = await buildHybridRepositoryHarness(
      FakeRemoteVideoSource([]),
      ports: HybridHarnessPorts(
        comments: comments,
      ),
    );
    final post = samplePost(nostrReference: reference);

    final topLevel = await harness.comments.publishComment(
      post: post,
      content: 'First',
    );
    final reply = await harness.comments.publishComment(
      post: post,
      content: 'Reply',
      replyTo: topLevel,
    );

    expect(await harness.comments.loadComments(post), [topLevel, reply]);
    expect(reply.parentCommentId, topLevel.id);
  });
}
