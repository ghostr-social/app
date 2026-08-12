import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('replaces a local compose draft with the shared video', (
    tester,
  ) async {
    final incoming = FakeIncomingVideoSharePort();
    addTearDown(incoming.close);
    final shared = SelectedMedia(
      path: '/tmp/replacement-share.mp4',
      source: MediaPickSource.externalShare,
      label: 'replacement-share.mp4',
      mimeType: VideoMimeType.fromFileName('replacement-share.mp4'),
    );
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: [samplePost()]),
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

    incoming.emit(IncomingVideoShareReady(shared));
    await tester.pumpAndSettle();

    expect(find.text(shared.path), findsOneWidget);
    expect(find.text(sampleMedia().path), findsNothing);
  });
}
