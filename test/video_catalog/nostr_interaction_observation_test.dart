import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_video_interactions.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fakes.dart';
import '../support/nostr_reference.dart';
import '../support/sample_data.dart';

void main() {
  test('marks only successfully hydrated interaction fields as observed',
      () async {
    final engagement = FakeNostrEngagementPort()
      ..loadFailure = const AppFailure('engagement unavailable');
    final interactions = NostrVideoInteractions(
      engagement,
      FakeNostrCommentsPort(),
      RecordingFailureReporter(),
    );
    final post = samplePost(nostrReference: nostrReference());

    final hydrated = await interactions.hydrate(post);

    expect(
      hydrated.metrics.likeObservation,
      VideoMetricObservation.unobserved,
    );
    expect(
      hydrated.metrics.commentObservation,
      VideoMetricObservation.observed,
    );
  });
}
