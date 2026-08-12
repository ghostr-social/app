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
  testWidgets('releases an active shared upload only after it finishes', (
    tester,
  ) async {
    final incoming = FakeIncomingVideoSharePort();
    final publishing = PendingVideoPublishingRepository();
    addTearDown(incoming.close);
    final shared = SelectedMedia(
      path: '/tmp/active-shared-upload.mp4',
      source: MediaPickSource.externalShare,
      label: 'active-shared-upload.mp4',
      mimeType: VideoMimeType.fromFileName('active-shared-upload.mp4'),
    );
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      publishing: publishing,
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: [samplePost()]),
      device: FakeDeviceDependencies(incomingVideoShares: incoming),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
    incoming.emit(IncomingVideoShareReady(shared));
    await tester.pumpAndSettle();
    await tester.ensureVisible(find.text('Publish'));
    await tester.tap(find.text('Publish'));
    await tester.pump();

    await tester.pumpWidget(const SizedBox());
    await tester.pump();
    expect(incoming.releasedMedia, isEmpty);

    publishing.result.complete(samplePost());
    await tester.pumpAndSettle();
    expect(incoming.releasedMedia, [shared]);
  });
}
