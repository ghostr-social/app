import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';
import 'package:ghostr/features/compose/presentation/compose_cubit.dart';

import '../support/fakes.dart';
import '../support/pending_video_publishing_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('keeps the publishing draft when another app shares a video', () async {
    final publishing = PendingVideoPublishingRepository();
    final cubit = ComposeCubit(
      buildComposeDependencies(
        publishing: publishing,
        activity: FakeActivityRepository(),
        picker: FakeMediaPickerPort(galleryMedia: sampleMedia()),
      ),
    );
    addTearDown(cubit.close);
    await cubit.chooseFromGallery();
    final publish = cubit.publish(sampleSession(), 'Publishing');

    cubit.acceptSharedVideo(_sharedVideo());

    expect(cubit.state.media?.path, sampleMedia().path);
    publishing.result.complete(samplePost());
    await publish;
  });
}

SelectedMedia _sharedVideo() => SelectedMedia(
  path: '/tmp/new-share.mp4',
  source: MediaPickSource.externalShare,
  label: 'new-share.mp4',
  mimeType: VideoMimeType.fromFileName('new-share.mp4'),
);
