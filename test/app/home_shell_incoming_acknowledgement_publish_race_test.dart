import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';

import '../support/fakes.dart';
import '../support/pending_video_publishing_repository.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('waits if publishing starts during share acknowledgement', (
    tester,
  ) async {
    final acknowledgement = Completer<void>();
    final incoming = FakeIncomingVideoSharePort()
      ..acknowledgeFuture = acknowledgement.future;
    final publishing = PendingVideoPublishingRepository();
    addTearDown(incoming.close);
    final shared = SelectedMedia(
      path: '/tmp/ack-race.mp4',
      source: MediaPickSource.externalShare,
      label: 'ack-race.mp4',
      mimeType: VideoMimeType.fromFileName('ack-race.mp4'),
    );
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      publishing: publishing,
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
    await tester.pump();
    await tester.ensureVisible(find.text('Publish'));
    await tester.tap(find.text('Publish'));
    acknowledgement.complete();
    await tester.pump();

    expect(find.text('Publishing...'), findsOneWidget);
    expect(incoming.releasedMedia, isEmpty);
    publishing.result.complete(samplePost());
    await tester.pumpAndSettle();

    expect(find.text(shared.path), findsOneWidget);
  });
}
