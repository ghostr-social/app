import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('one long upward gesture advances exactly one video', (
    tester,
  ) async {
    final repository = FakeVideoCatalogRepository(
      forYouFeed: List.generate(4, (index) => samplePost(id: 'post-$index')),
    );
    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();
    final pageView = find.byType(PageView);
    final height = tester.getSize(pageView).height;

    final gesture = await tester.startGesture(tester.getCenter(pageView));
    for (var step = 0; step < 4; step += 1) {
      await gesture.moveBy(Offset(0, -height * 0.2));
      await tester.pump(const Duration(milliseconds: 50));
    }
    await gesture.up();
    await tester.pumpAndSettle();

    final cubit = BlocProvider.of<FeedCubit>(tester.element(pageView));
    expect((cubit.state as FeedLoaded).activeIndex, 1);
  });
}
