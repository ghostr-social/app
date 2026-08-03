import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/media_picker_port.dart';
import 'package:ghostr/core/media/media_picker_capabilities.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/features/compose/presentation/compose_cubit.dart';

import '../support/fakes.dart';

void main() {
  test('ignores a media picker completion after disposal', () async {
    final picker = _PendingPicker();
    final cubit = ComposeCubit(buildComposeDependencies(
      publishing: FakeVideoCatalogRepository(forYouFeed: []),
      activity: FakeActivityRepository(),
      picker: picker,
    ));

    final selection = cubit.chooseFromGallery();
    final completion = expectLater(selection, completes);
    await cubit.close();
    picker.pending.complete(null);

    await completion;
  });
}

class _PendingPicker implements MediaPickerPort {
  final pending = Completer<SelectedMedia?>();

  @override
  MediaPickerCapabilities get capabilities =>
      const MediaPickerCapabilities.allSupported();

  @override
  Future<SelectedMedia?> pickFromGallery() => pending.future;

  @override
  Future<SelectedMedia?> captureVideo() async => null;

  @override
  Future<SelectedMedia?> recoverLostVideo() async => null;
}
