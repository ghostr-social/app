import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/features/compose/presentation/compose_cubit.dart';

import '../support/fakes.dart';

void main() {
  test('uses an app-safe message for an unexpected media picker error',
      () async {
    final cubit = ComposeCubit(buildComposeDependencies(
      publishing: FakeVideoCatalogRepository(forYouFeed: []),
      activity: FakeActivityRepository(),
      picker: _UnexpectedPicker(),
      clock: DateTime.now,
    ));
    addTearDown(cubit.close);

    await cubit.chooseFromGallery();

    expect(cubit.state.errorMessage, 'Could not open this video.');
  });
}

class _UnexpectedPicker extends FakeMediaPickerPort {
  @override
  Future<SelectedMedia?> pickFromGallery() {
    throw StateError('picker unavailable');
  }
}
