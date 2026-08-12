import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/profile_video_grid.dart';

import '../support/sample_data.dart';

void main() {
  testWidgets('a profile video grid without an opener stays inert',
      (tester) async {
    final posts = [samplePost(id: 'clip-1', caption: 'Quiet clip')];
    await tester.pumpWidget(
      MaterialApp(home: ListView(children: [ProfileVideoGrid(posts: posts)])),
    );

    final tile = find.byKey(ProfileVideoGrid.tileKey(posts.first.id));
    expect(
      tester.getSemantics(tile),
      isNot(isSemantics(hasTapAction: true)),
    );

    await tester.tap(tile, warnIfMissed: false);
    expect(find.text('Quiet clip'), findsOneWidget);
  });
}
