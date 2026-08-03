import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/media_picker_port.dart';
import 'package:ghostr/core/media/media_picker_capabilities.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/features/compose/presentation/compose_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('allows only one media picker operation at a time', () async {
    final picker = _PendingPicker();
    final cubit = ComposeCubit(buildComposeDependencies(
      publishing: FakeVideoCatalogRepository(forYouFeed: []),
      activity: FakeActivityRepository(),
      picker: picker,
    ));

    final gallery = cubit.chooseFromGallery();
    await cubit.captureVideo();
    picker.gallery.complete(sampleMedia());
    await gallery;

    expect(picker.cameraCount, 0);
    expect(cubit.state.media?.path, sampleMedia().path);
    await cubit.close();
  });
}

class _PendingPicker implements MediaPickerPort {
  final gallery = Completer<SelectedMedia?>();
  int cameraCount = 0;

  @override
  MediaPickerCapabilities get capabilities =>
      const MediaPickerCapabilities.allSupported();

  @override
  Future<SelectedMedia?> pickFromGallery() => gallery.future;

  @override
  Future<SelectedMedia?> captureVideo() async {
    cameraCount += 1;
    return sampleMedia();
  }

  @override
  Future<SelectedMedia?> recoverLostVideo() async => null;
}
