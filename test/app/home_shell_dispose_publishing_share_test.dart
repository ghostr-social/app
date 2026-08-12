import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';

import '../support/fakes.dart';
import '../support/pending_video_publishing_repository.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('keeps a pending share until the active upload finishes', (
    tester,
  ) async {
    final incoming = FakeIncomingVideoSharePort();
    final publishing = PendingVideoPublishingRepository();
    addTearDown(incoming.close);
    final shared = SelectedMedia(
      path: '/tmp/waiting-share.mp4',
      source: MediaPickSource.externalShare,
      label: 'waiting-share.mp4',
      mimeType: VideoMimeType.fromFileName('waiting-share.mp4'),
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
    await tester.ensureVisible(find.text('Publish'));
    await tester.tap(find.text('Publish'));
    await tester.pump();
    incoming.emit(IncomingVideoShareReady(shared));
    await tester.pump();

    await tester.pumpWidget(const SizedBox());
    expect(incoming.releasedMedia, [shared]);
    publishing.result.complete(samplePost());
    await tester.pumpAndSettle();

    expect(
      incoming.releasedMedia.where(
        (media) => media.source == MediaPickSource.externalShare,
      ),
      [shared],
    );
  });
}
