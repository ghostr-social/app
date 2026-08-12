import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/app/router/app_router.dart';
import 'package:ghostr/app/profile_route_request.dart';
import 'package:ghostr/features/video_catalog/presentation/discovery_feed_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_screen.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('a discovery feed route chains into tags and profiles', (
    tester,
  ) async {
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(caption: 'Tagged clip')],
    );
    final controllers = AppControllerFactory(
      buildFakeDependencies(
        session: sampleSession(),
        catalogRepository: repository,
      ),
    );
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) => ElevatedButton(
            onPressed: () => Navigator.of(context).push(
              AppRouter.discoveryFeed(
                DiscoveryFeedRouteRequest(
                  session: sampleSession(),
                  query: '#dance',
                  controllers: controllers,
                  onSignedOut: () {},
                ),
              ),
            ),
            child: const Text('open'),
          ),
        ),
      ),
    );
    await tester.tap(find.text('open'));
    await tester.pumpAndSettle();
    expect(find.widgetWithText(AppBar, '#dance'), findsOneWidget);

    final request = tester
        .widget<DiscoveryFeedScreen>(find.byType(DiscoveryFeedScreen))
        .request;
    final hashtagRoute = request.onOpenHashtag('#music');
    await tester.pumpAndSettle();
    expect(find.widgetWithText(AppBar, '#music'), findsOneWidget);
    tester.state<NavigatorState>(find.byType(Navigator)).pop();
    await tester.pumpAndSettle();
    await hashtagRoute;

    final profileRoute = request.onOpenProfile(sampleCreator().id);
    await tester.pumpAndSettle();
    expect(find.byType(ProfileScreen), findsOneWidget);
    tester.state<NavigatorState>(find.byType(Navigator)).pop();
    await tester.pumpAndSettle();
    await profileRoute;
  });
}
