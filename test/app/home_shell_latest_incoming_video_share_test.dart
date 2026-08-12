import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('keeps the latest video when multiple shares arrive', (
    tester,
  ) async {
    final incoming = FakeIncomingVideoSharePort();
    addTearDown(incoming.close);
    final recovery = Completer<SelectedMedia?>();
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
    incoming.emit(IncomingVideoShareReady(_sharedVideo('first')));
    await tester.pump();
    incoming.emit(IncomingVideoShareReady(_sharedVideo('latest')));
    await tester.pump();
    recovery.complete();
    await tester.pumpAndSettle();

    expect(find.text('/tmp/latest.mp4'), findsOneWidget);
    expect(find.text('/tmp/first.mp4'), findsNothing);
    expect(incoming.releasedMedia.map((media) => media.label), ['first.mp4']);
  });
}

SelectedMedia _sharedVideo(String name) => SelectedMedia(
  path: '/tmp/$name.mp4',
  source: MediaPickSource.externalShare,
  label: '$name.mp4',
  mimeType: VideoMimeType.fromFileName('$name.mp4'),
);
