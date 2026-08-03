import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/compose/domain/publish_video_workflow.dart';
import 'package:ghostr/features/publish/domain/video_publication.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('aggregates catalog and activity warnings after publication', () async {
    final workflow = DefaultPublishVideoWorkflow(
      publishing: FakeVideoCatalogRepository(
        forYouFeed: [],
        cacheStatus: VideoPublicationCacheStatus.unavailable,
      ),
      activity: _FailingActivityRepository(),
      clock: () => DateTime.utc(2026, 8, 3),
      failureReporter: RecordingFailureReporter(),
    );

    final outcome = await workflow.publish(
      session: sampleSession(),
      media: sampleMedia(),
      rawCaption: 'Published once',
    );

    expect(outcome.warnings, {
      PublishVideoWarning.localCatalogUnavailable,
      PublishVideoWarning.localActivityUnavailable,
    });
  });
}

class _FailingActivityRepository implements ActivityRepository {
  @override
  ActivityRepository snapshotForActiveAccount() => this;

  @override
  Future<List<ActivityItem>> load() async => const [];

  @override
  Future<void> record(ActivityItem item) async {
    throw StateError('preferences unavailable');
  }
}
