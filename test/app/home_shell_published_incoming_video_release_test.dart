import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('releases an imported video after it is published', (
    tester,
  ) async {
    final incoming = FakeIncomingVideoSharePort();
    addTearDown(incoming.close);
    final media = _sharedVideo();
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: [samplePost()]),
      device: FakeDeviceDependencies(incomingVideoShares: incoming),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
    incoming.emit(IncomingVideoShareReady(media));
    await tester.pumpAndSettle();

    await tester.ensureVisible(find.text('Publish'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Publish'));
    await tester.pumpAndSettle();

    expect(incoming.releasedMedia, [media]);
  });
}

SelectedMedia _sharedVideo() => SelectedMedia(
  path: '/tmp/published-share.mp4',
  source: MediaPickSource.externalShare,
  label: 'published-share.mp4',
  mimeType: VideoMimeType.fromFileName('published-share.mp4'),
);
