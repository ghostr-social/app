import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('a cancelled swipe does not affect the next feed swipe', (
    tester,
  ) async {
    final repository = FakeVideoCatalogRepository(
      forYouFeed: List.generate(3, (index) => samplePost(id: 'post-$index')),
    );
    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();
    final pageView = find.byType(PageView);
    final height = tester.getSize(pageView).height;

    final cancelled = await tester.startGesture(tester.getCenter(pageView));
    await cancelled.moveBy(Offset(0, -height * 0.1));
    await tester.pump();
    await cancelled.cancel();
    await tester.pumpAndSettle();

    final cubit = BlocProvider.of<FeedCubit>(tester.element(pageView));
    expect((cubit.state as FeedLoaded).activeIndex, 0);

    final next = await tester.startGesture(tester.getCenter(pageView));
    await next.moveBy(Offset(0, -height * 0.23));
    await next.up();
    await tester.pumpAndSettle();

    expect((cubit.state as FeedLoaded).activeIndex, 1);
  });
}
