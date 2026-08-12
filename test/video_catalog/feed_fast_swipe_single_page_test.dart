import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('one fast short swipe advances exactly one video', (
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
    await gesture.moveBy(Offset(0, -height * 0.23));
    await tester.pump(const Duration(milliseconds: 16));
    await gesture.up();
    await tester.pumpAndSettle();

    final cubit = BlocProvider.of<FeedCubit>(tester.element(pageView));
    expect((cubit.state as FeedLoaded).activeIndex, 1);
  });
}
