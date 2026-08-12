import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('releases a shared video when native acknowledgement fails', (
    tester,
  ) async {
    final incoming = FakeIncomingVideoSharePort()
      ..acknowledgeFailure = StateError('native failure');
    addTearDown(incoming.close);
    final media = SelectedMedia(
      path: '/tmp/unacknowledged-share.mp4',
      source: MediaPickSource.externalShare,
      label: 'unacknowledged-share.mp4',
      mimeType: VideoMimeType.fromFileName('unacknowledged-share.mp4'),
    );
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: [samplePost()]),
      device: FakeDeviceDependencies(incomingVideoShares: incoming),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    incoming.emit(IncomingVideoShareReady(media));
    await tester.pumpAndSettle();

    expect(incoming.releasedMedia, [media]);
    expect(find.text(media.path), findsNothing);
    expect(find.text('Could not open the shared video.'), findsOneWidget);
  });
}
