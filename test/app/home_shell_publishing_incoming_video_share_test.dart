import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';

import '../support/fakes.dart';
import '../support/pending_video_publishing_repository.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('opens the latest shared video after publishing finishes', (
    tester,
  ) async {
    final incoming = FakeIncomingVideoSharePort();
    final publishing = PendingVideoPublishingRepository();
    addTearDown(incoming.close);
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: [samplePost()]),
      publishing: publishing,
      device: FakeDeviceDependencies(
        incomingVideoShares: incoming,
        mediaPicker: FakeMediaPickerPort(galleryMedia: sampleMedia()),
      ),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Create'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Choose from library'));
    await tester.pumpAndSettle();
    await tester.ensureVisible(find.text('Publish'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Publish'));
    await tester.pump();

    incoming.emit(IncomingVideoShareReady(_sharedVideo('first')));
    await tester.pump();
    incoming.emit(IncomingVideoShareReady(_sharedVideo('latest')));
    await tester.pump();
    publishing.result.complete(samplePost());
    await tester.pumpAndSettle();

    expect(find.text('/tmp/latest-while-publishing.mp4'), findsOneWidget);
    expect(
      incoming.releasedMedia.map((media) => media.label),
      contains('first.mp4'),
    );
  });
}

SelectedMedia _sharedVideo(String name) => SelectedMedia(
  path: '/tmp/$name-while-publishing.mp4',
  source: MediaPickSource.externalShare,
  label: '$name.mp4',
  mimeType: VideoMimeType.fromFileName('$name.mp4'),
);
