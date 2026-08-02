import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/compose/presentation/compose_cubit.dart';

import '../support/fakes.dart';
import '../support/pending_video_publishing_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('ignores a second publish intent while publishing', () async {
    final publishing = PendingVideoPublishingRepository();
    final cubit = ComposeCubit(buildComposeDependencies(
      publishing: publishing,
      activity: FakeActivityRepository(),
      picker: FakeMediaPickerPort(galleryMedia: sampleMedia()),
      clock: () => DateTime(2026, 8, 2),
    ));
    await cubit.chooseFromGallery();

    final first = cubit.publish(sampleSession(), 'Pending clip');
    final second = await cubit.publish(sampleSession(), 'Duplicate clip');

    expect(second, isFalse);
    expect(publishing.publishCount, 1);
    publishing.result.complete(samplePost());
    await first;
    await cubit.close();
  });
}
