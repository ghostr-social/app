import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets(
    'releases the active shared video when its session shell closes',
    (tester) async {
      final incoming = FakeIncomingVideoSharePort();
      addTearDown(incoming.close);
      final media = SelectedMedia(
        path: '/tmp/account-shared.mp4',
        source: MediaPickSource.externalShare,
        label: 'account-shared.mp4',
        mimeType: VideoMimeType.fromFileName('account-shared.mp4'),
      );
      final dependencies = buildFakeDependencies(
        session: sampleSession(),
        catalogRepository: FakeVideoCatalogRepository(
          forYouFeed: [samplePost()],
        ),
        device: FakeDeviceDependencies(incomingVideoShares: incoming),
      );
      await tester.pumpWidget(buildTestApp(dependencies));
      await tester.pumpAndSettle();
      incoming.emit(IncomingVideoShareReady(media));
      await tester.pumpAndSettle();

      await tester.pumpWidget(const SizedBox());
      await tester.pumpAndSettle();

      expect(incoming.releasedMedia, [media]);
    },
  );
}
