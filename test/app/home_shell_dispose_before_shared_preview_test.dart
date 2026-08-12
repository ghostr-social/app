import 'dart:async';

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
    'releases an accepted share when the shell closes before preview',
    (tester) async {
      final acknowledgement = Completer<void>();
      final incoming = FakeIncomingVideoSharePort()
        ..acknowledgeFuture = acknowledgement.future;
      addTearDown(incoming.close);
      final media = _sharedVideo();
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
      await tester.pump();
      acknowledgement.complete();
      await tester.idle();
      expect(incoming.acknowledgedMedia, [media]);

      await tester.pumpWidget(const SizedBox());
      await tester.pumpAndSettle();

      expect(incoming.releasedMedia, [media]);
    },
  );
}

SelectedMedia _sharedVideo() => SelectedMedia(
  path: '/tmp/accepted-before-preview.mp4',
  source: MediaPickSource.externalShare,
  label: 'accepted-before-preview.mp4',
  mimeType: VideoMimeType.fromFileName('accepted-before-preview.mp4'),
);
