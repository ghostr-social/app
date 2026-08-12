import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';
import 'package:ghostr/features/compose/presentation/compose_cubit.dart';

import '../support/fakes.dart';

void main() {
  test('accepts an externally shared video as the compose selection', () {
    final media = SelectedMedia(
      path: '/tmp/shared-video.mp4',
      source: MediaPickSource.externalShare,
      label: 'shared-video.mp4',
      mimeType: VideoMimeType.fromFileName('shared-video.mp4'),
    );
    final cubit = ComposeCubit(
      buildComposeDependencies(
        publishing: FakeVideoCatalogRepository(forYouFeed: []),
        activity: FakeActivityRepository(),
        picker: FakeMediaPickerPort(),
      ),
    );
    addTearDown(cubit.close);

    cubit.acceptSharedVideo(media);

    expect(cubit.state.media, same(media));
    expect(cubit.state.media?.source, MediaPickSource.externalShare);
    expect(cubit.state.isBusy, isFalse);
  });
}
