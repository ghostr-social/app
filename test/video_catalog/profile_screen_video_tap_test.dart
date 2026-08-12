import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/profile_video_grid.dart';

import '../support/fakes.dart';
import '../support/profile_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('a profile screen hands tapped videos to the opener',
      (tester) async {
    final creator = sampleCreator();
    final clip = samplePost(id: 'clip-1', caption: 'Shelf clip', creator: creator);
    final opened = <VideoPost>[];
    final repository = FakeVideoCatalogRepository(
      forYouFeed: const [],
      feed: FakeFeedScenario(
        profiles: {
          creator.id: sampleProfileDetails(profile: creator, posts: [clip]),
        },
      ),
    );
    await tester.pumpWidget(
      profileScreenHarness(
        profile: repository,
        viewer: sampleCreator(id: 'viewer-1'),
        profileId: creator.id,
        onOpenVideo: opened.add,
      ),
    );
    await tester.pumpAndSettle();

    await tester.ensureVisible(find.byKey(ProfileVideoGrid.tileKey(clip.id)));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(ProfileVideoGrid.tileKey(clip.id)));

    expect(opened, [clip]);
  });
}
