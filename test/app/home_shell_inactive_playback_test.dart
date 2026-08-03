import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/recording_video_playback_port.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('switching tabs deactivates hidden video surfaces',
      (tester) async {
    final feedPost = samplePost();
    final draft = sampleMedia();
    final playback = RecordingVideoPlaybackPort();
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: [feedPost]),
      device: FakeDeviceDependencies(
        mediaPicker: FakeMediaPickerPort(galleryMedia: draft),
        playback: playback,
      ),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
    expect(playback.activity[feedPost.media.debugLabel]!.last, isTrue);

    await tester.tap(find.text('Create'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Choose from library'));
    await tester.pumpAndSettle();
    expect(playback.activity[feedPost.media.debugLabel]!.last, isFalse);
    expect(playback.activity[draft.path]!.last, isTrue);

    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();
    expect(playback.activity[draft.path]!.last, isFalse);
  });
}
