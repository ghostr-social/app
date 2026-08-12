import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_mime_type.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('closes a covering route before opening a shared video', (
    tester,
  ) async {
    final post = samplePost();
    final incoming = FakeIncomingVideoSharePort();
    addTearDown(incoming.close);
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: [post],
        feed: FakeFeedScenario(
          profiles: {post.creator.id: sampleProfileDetails()},
        ),
      ),
      device: FakeDeviceDependencies(incomingVideoShares: incoming),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('Open profile'));
    await tester.pumpAndSettle();
    expect(find.text('Follow'), findsOneWidget);

    incoming.emit(IncomingVideoShareReady(_sharedVideo()));
    await tester.pumpAndSettle();

    expect(find.text('/tmp/routed-shared-video.mp4'), findsOneWidget);
    expect(find.text('Follow'), findsNothing);
  });
}

SelectedMedia _sharedVideo() => SelectedMedia(
  path: '/tmp/routed-shared-video.mp4',
  source: MediaPickSource.externalShare,
  label: 'routed-shared-video.mp4',
  mimeType: VideoMimeType.fromFileName('routed-shared-video.mp4'),
);
