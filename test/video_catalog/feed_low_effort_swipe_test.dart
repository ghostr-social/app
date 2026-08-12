import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('a short slow upward swipe advances one page within 250ms', (
    tester,
  ) async {
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [
        samplePost(id: 'first'),
        samplePost(id: 'second'),
        samplePost(id: 'third'),
      ],
    );
    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();

    final pageView = find.byType(PageView);
    final swipeDistance = tester.getSize(pageView).height * 0.23;
    final gesture = await tester.startGesture(tester.getCenter(pageView));
    for (var step = 0; step < 4; step += 1) {
      await gesture.moveBy(Offset(0, -swipeDistance / 4));
      await tester.pump(const Duration(milliseconds: 75));
    }
    await tester.pump(const Duration(milliseconds: 50));
    await gesture.up();
    await tester.pump(const Duration(milliseconds: 250));

    final cubit = BlocProvider.of<FeedCubit>(tester.element(pageView));
    expect((cubit.state as FeedLoaded).activeIndex, 1);
    final controller = tester.widget<PageView>(pageView).controller!;
    expect(controller.page, 1);
  });
}
