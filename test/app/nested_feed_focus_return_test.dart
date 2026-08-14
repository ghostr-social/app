import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/app/profile_route_request.dart';
import 'package:ghostr/app/router/app_router.dart';
import 'package:ghostr/features/video_catalog/presentation/discovery_feed_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/fake_feed_focus_port.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('popping a nested feed restores its parent feed focus', (
    tester,
  ) async {
    final posts = [samplePost(id: 'first'), samplePost(id: 'second')];
    final focus = FakeFeedFocusPort();
    final controllers = AppControllerFactory(
      buildFakeDependencies(
        catalogRepository: FakeVideoCatalogRepository(forYouFeed: posts),
      ),
      feedFocus: focus,
    );
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) {
            return TextButton(
              onPressed: () => Navigator.of(context).push(
                AppRouter.discoveryFeed(
                  DiscoveryFeedRouteRequest(
                    session: sampleSession(),
                    query: '#parent',
                    controllers: controllers,
                    onSignedOut: () {},
                  ),
                ),
              ),
              child: const Text('open'),
            );
          },
        ),
      ),
    );
    await tester.tap(find.text('open'));
    await tester.pumpAndSettle();
    final parent = find.byType(DiscoveryFeedScreen);
    tester.element(parent).read<FeedCubit>().pageChanged(1);
    final returned = tester
        .widget<DiscoveryFeedScreen>(parent)
        .request
        .onOpenHashtag('#nested');
    await tester.pumpAndSettle();

    expect(focus.focuses.last.currentIndex, 0);
    await tester.pageBack();
    await tester.pumpAndSettle();
    await returned;

    expect(focus.focuses.last.currentIndex, 1);
    expect(focus.focuses.last.current.id.value, 'second');
  });
}
