import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/compose/presentation/compose_cubit.dart';

import '../support/fakes.dart';
import '../support/pending_video_publishing_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('ignores media selection while a publish is pending', () async {
    final picker = FakeMediaPickerPort(galleryMedia: sampleMedia());
    final publishing = PendingVideoPublishingRepository();
    final cubit = ComposeCubit(buildComposeDependencies(
      publishing: publishing,
      activity: FakeActivityRepository(),
      picker: picker,
      clock: () => DateTime(2026, 8, 2),
    ));
    await cubit.chooseFromGallery();

    final publish = cubit.publish(sampleSession(), 'Pending clip');
    await cubit.chooseFromGallery();

    expect(picker.galleryPickCount, 1);
    publishing.result.complete(samplePost());
    await publish;
    await cubit.close();
  });
}
