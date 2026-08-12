import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/home_tab.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('opens Create with a warm shared video ready to publish', (
    tester,
  ) async {
    final incoming = FakeIncomingVideoSharePort();
    addTearDown(incoming.close);
    final media = _sharedVideo();
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: [samplePost()]),
      device: FakeDeviceDependencies(incomingVideoShares: incoming),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    incoming.emit(IncomingVideoShareReady(media));
    await tester.pumpAndSettle();

    expect(find.text(media.path), findsOneWidget);
    final navigation = tester.widget<BottomNavigationBar>(
      find.byType(BottomNavigationBar),
    );
    expect(navigation.currentIndex, HomeTab.values.indexOf(HomeTab.create));
    final publish = tester.widget<ElevatedButton>(
      find.widgetWithText(ElevatedButton, 'Publish'),
    );
    expect(publish.onPressed, isNotNull);
    expect(incoming.acknowledgedMedia, [media]);
  });
}

SelectedMedia _sharedVideo() => SelectedMedia(
  path: '/tmp/shared-from-whatsapp.mp4',
  source: MediaPickSource.externalShare,
  label: 'shared-from-whatsapp.mp4',
  mimeType: VideoMimeType.fromFileName('shared-from-whatsapp.mp4'),
);
