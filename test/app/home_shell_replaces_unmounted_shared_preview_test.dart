import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('releases an accepted share replaced before its preview mounts', (
    tester,
  ) async {
    final incoming = FakeIncomingVideoSharePort();
    addTearDown(incoming.close);
    final first = _sharedVideo('first');
    final latest = _sharedVideo('latest');
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: [samplePost()]),
      device: FakeDeviceDependencies(incomingVideoShares: incoming),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    incoming.emit(IncomingVideoShareReady(first));
    await tester.idle();
    incoming.emit(IncomingVideoShareReady(latest));
    await tester.idle();
    await tester.pump();

    expect(find.text(latest.path), findsOneWidget);
    expect(find.text(first.path), findsNothing);
    expect(incoming.releasedMedia, [first]);
  });
}

SelectedMedia _sharedVideo(String name) => SelectedMedia(
  path: '/tmp/$name.mp4',
  source: MediaPickSource.externalShare,
  label: '$name.mp4',
  mimeType: VideoMimeType.fromFileName('$name.mp4'),
);
