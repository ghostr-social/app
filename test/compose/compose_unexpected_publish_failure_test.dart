import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/features/compose/presentation/compose_cubit.dart';
import 'package:ghostr/features/publish/domain/video_publication.dart';
import 'package:ghostr/features/session/domain/user_session.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('uses an app-safe message for an unexpected publishing error', () async {
    final cubit = ComposeCubit(buildComposeDependencies(
      publishing: _UnexpectedPublishingRepository(),
      activity: FakeActivityRepository(),
      picker: FakeMediaPickerPort(galleryMedia: sampleMedia()),
      clock: DateTime.now,
    ));
    addTearDown(cubit.close);
    await cubit.chooseFromGallery();

    final published = await cubit.publish(sampleSession(), 'Caption');

    expect(published, isFalse);
    expect(cubit.state.errorMessage, 'Could not publish this video.');
  });
}

class _UnexpectedPublishingRepository extends FakeVideoCatalogRepository {
  _UnexpectedPublishingRepository() : super(forYouFeed: []);

  @override
  Future<VideoPublication> publish({
    required UserSession session,
    required SelectedMedia media,
    required String caption,
  }) {
    throw StateError('publisher unavailable');
  }
}
