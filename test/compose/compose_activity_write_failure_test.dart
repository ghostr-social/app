import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_repository.dart';
import 'package:ghostr/features/compose/presentation/compose_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  final reporter = RecordingFailureReporter();
  blocTest<ComposeCubit, ComposeState>(
    'keeps a successful publish when local activity recording fails',
    build: () => ComposeCubit(buildComposeDependencies(
      publishing: FakeVideoCatalogRepository(forYouFeed: []),
      activity: _FailingActivityRepository(),
      picker: FakeMediaPickerPort(galleryMedia: sampleMedia()),
      clock: () => DateTime(2026, 8, 2),
      failureReporter: reporter,
    )),
    act: (cubit) async {
      await cubit.chooseFromGallery();
      await cubit.publish(sampleSession(), 'A Nostr clip');
    },
    verify: (cubit) {
      expect(
        cubit.state.notice,
        'Published, but local activity history could not be updated.',
      );
      expect(reporter.sources, ['DefaultPublishVideoWorkflow.record']);
    },
  );
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
