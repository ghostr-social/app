import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('opens a cold-start shared video after session restoration', (
    tester,
  ) async {
    final media = SelectedMedia(
      path: '/tmp/cold-shared-video.mp4',
      source: MediaPickSource.externalShare,
      label: 'cold-shared-video.mp4',
      mimeType: VideoMimeType.fromFileName('cold-shared-video.mp4'),
    );
    final incoming = FakeIncomingVideoSharePort(
      initialEvent: IncomingVideoShareReady(media),
    );
    addTearDown(incoming.close);
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: [samplePost()]),
      device: FakeDeviceDependencies(incomingVideoShares: incoming),
    );

    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    expect(find.text(media.path), findsOneWidget);
    expect(find.text('No draft selected'), findsNothing);
  });
}
