import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/engagement/domain/video_engagement.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_video_interactions.dart';

import '../support/fake_nostr_comments_port.dart';
import '../support/fake_nostr_engagement_port.dart';
import '../support/nostr_reference.dart';
import '../support/recording_failure_reporter.dart';
import '../support/sample_data.dart';

void main() {
  test('retains snapshot metrics when hydration exceeds its total deadline',
      () async {
    final reporter = RecordingFailureReporter();
    final interactions = NostrVideoInteractions(
      _HangingEngagement(),
      FakeNostrCommentsPort(),
      reporter,
      hydrationTimeout: const Duration(milliseconds: 1),
    );
    final post = samplePost(nostrReference: nostrReference());

    final hydrated = await interactions.hydrate(post);

    expect(hydrated.likeCount, post.likeCount);
    expect(hydrated.commentCount, post.commentCount);
    expect(reporter.sources, ['NostrVideoInteractions.hydrateAll']);
  });
}

class _HangingEngagement extends FakeNostrEngagementPort {
  @override
  Future<Map<NostrEventId, VideoEngagement>> loadBatch(
    List<NostrEventReference> references,
  ) {
    return Completer<Map<NostrEventId, VideoEngagement>>().future;
  }
}
