import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/compose/domain/publish_video_workflow.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('publishes with a filename fallback and records activity', () async {
    final publishing = FakeVideoCatalogRepository(forYouFeed: []);
    final activity = FakeActivityRepository();
    final reporter = RecordingFailureReporter();
    final workflow = DefaultPublishVideoWorkflow(
      publishing: publishing,
      activity: activity,
      clock: () => DateTime.utc(2026, 8, 2),
      failureReporter: reporter,
    );

    final notice = await workflow.publish(
      session: sampleSession(),
      media: sampleMedia(),
      rawCaption: '  ',
    );

    expect(publishing.forYouFeed.single.caption, sampleMedia().label);
    expect((await activity.load()).single.body, sampleMedia().label);
    expect(notice, PublishVideoOutcome.published);
    expect(reporter.sources, isEmpty);
  });
}
