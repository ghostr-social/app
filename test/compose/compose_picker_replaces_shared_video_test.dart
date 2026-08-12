import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';
import 'package:ghostr/features/compose/presentation/compose_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('releases a shared video replaced from the gallery', () async {
    final released = <SelectedMedia>[];
    final picker = FakeMediaPickerPort(galleryMedia: sampleMedia());
    final cubit = ComposeCubit(
      buildComposeDependencies(
        publishing: FakeVideoCatalogRepository(forYouFeed: []),
        activity: FakeActivityRepository(),
        picker: picker,
      ),
    );
    cubit.bindPreviewRelease((media) async => released.add(media));
    addTearDown(cubit.close);
    final shared = SelectedMedia(
      path: '/tmp/shared.mp4',
      source: MediaPickSource.externalShare,
      label: 'shared.mp4',
      mimeType: VideoMimeType.fromFileName('shared.mp4'),
    );
    cubit.acceptSharedVideo(shared);

    await cubit.chooseFromGallery();
    await cubit.releasePreviewMedia(shared);

    expect(cubit.state.media?.path, sampleMedia().path);
    expect(released, [shared]);
  });
}
