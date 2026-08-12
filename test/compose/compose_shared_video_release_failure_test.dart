import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';
import 'package:ghostr/features/compose/presentation/compose_cubit.dart';

import '../support/fakes.dart';

void main() {
  test('contains a preview cleanup failure at the platform boundary', () async {
    final cubit = ComposeCubit(
      buildComposeDependencies(
        publishing: FakeVideoCatalogRepository(forYouFeed: []),
        activity: FakeActivityRepository(),
        picker: FakeMediaPickerPort(),
      ),
    );
    addTearDown(cubit.close);
    cubit.bindPreviewRelease((_) => Future<void>.error(StateError('disk')));
    final shared = SelectedMedia(
      path: '/tmp/shared.mp4',
      source: MediaPickSource.externalShare,
      label: 'shared.mp4',
      mimeType: VideoMimeType.fromFileName('shared.mp4'),
    );

    await cubit.releasePreviewMedia(shared);
  });
}
