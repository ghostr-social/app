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
  testWidgets('releases a pending share when its session shell closes', (
    tester,
  ) async {
    final recovery = Completer<SelectedMedia?>();
    final incoming = FakeIncomingVideoSharePort();
    addTearDown(incoming.close);
    final media = SelectedMedia(
      path: '/tmp/pending-account-share.mp4',
      source: MediaPickSource.externalShare,
      label: 'pending-account-share.mp4',
      mimeType: VideoMimeType.fromFileName('pending-account-share.mp4'),
    );
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: [samplePost()]),
      device: FakeDeviceDependencies(
        incomingVideoShares: incoming,
        mediaPicker: FakeMediaPickerPort(recoveredMediaFuture: recovery.future),
      ),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Create'));
    await tester.pump();
    incoming.emit(IncomingVideoShareReady(media));
    await tester.pump();

    await tester.pumpWidget(const SizedBox());
    recovery.complete();
    await tester.pumpAndSettle();

    expect(incoming.releasedMedia, [media]);
  });
}
