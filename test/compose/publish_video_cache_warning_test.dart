import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/compose/domain/publish_video_workflow.dart';
import 'package:ghostr/features/publish/domain/video_publication.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('reports a local catalog warning after a completed publish', () async {
    final publishing = FakeVideoCatalogRepository(
      forYouFeed: [],
      cacheStatus: VideoPublicationCacheStatus.unavailable,
    );
    final workflow = DefaultPublishVideoWorkflow(
      publishing: publishing,
      activity: FakeActivityRepository(),
      clock: () => DateTime.utc(2026, 8, 3),
      failureReporter: RecordingFailureReporter(),
    );

    final outcome = await workflow.publish(
      session: sampleSession(),
      media: sampleMedia(),
      rawCaption: 'Published once',
    );

    expect(outcome.warnings, {PublishVideoWarning.localCatalogUnavailable});
  });
}
