import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/profile_video_grid.dart';

import '../support/sample_data.dart';

void main() {
  testWidgets('tapping a profile video hands the post to the opener',
      (tester) async {
    final posts = [
      samplePost(id: 'clip-1', caption: 'First clip'),
      samplePost(id: 'clip-2', caption: 'Second clip'),
    ];
    final opened = <VideoPost>[];
    await tester.pumpWidget(
      MaterialApp(
        home: ListView(
          children: [ProfileVideoGrid(posts: posts, onOpenVideo: opened.add)],
        ),
      ),
    );

    final tile = find.byKey(ProfileVideoGrid.tileKey(posts[1].id));
    expect(
      tester.getSemantics(tile),
      isSemantics(isButton: true, hasTapAction: true),
    );

    await tester.tap(tile);
    expect(opened, [posts[1]]);
  });
}
