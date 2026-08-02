import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/compose/presentation/compose_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  final activity = FakeActivityRepository();
  final picker = FakeMediaPickerPort(galleryMedia: sampleMedia());
  blocTest<ComposeCubit, ComposeState>(
    'publishes selected media and records device activity',
    build: () => ComposeCubit(buildComposeDependencies(
      publishing: FakeVideoCatalogRepository(forYouFeed: []),
      activity: activity,
      picker: picker,
      clock: () => DateTime(2026, 8, 2),
    )),
    act: (cubit) async {
      await cubit.chooseFromGallery();
      await cubit.publish(sampleSession(), 'A Nostr clip');
    },
    verify: (cubit) async {
      expect(cubit.state.media, isNull);
      expect(cubit.state.notice, 'Published to your Ghostr profile.');
      expect((await activity.load()).single.body, 'A Nostr clip');
    },
  );
}
