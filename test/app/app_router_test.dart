import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/app/router/app_router.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('pushes the typed profile route', (tester) async {
    final creator = sampleCreator();
    final catalog = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(creator: creator)],
      feed: FakeFeedScenario(profiles: {
        creator.id: sampleProfileDetails(profile: creator),
      }),
    );
    final controllers = AppControllerFactory(buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: catalog,
    ));
    await tester.pumpWidget(MaterialApp(
      home: Builder(builder: (context) {
        return ElevatedButton(
          onPressed: () => Navigator.of(context).push(AppRouter.profile(
            session: sampleSession(),
            profileId: creator.id,
            controllers: controllers,
            onSignedOut: () {},
          )),
          child: const Text('Open profile'),
        );
      }),
    ));

    await tester.tap(find.text('Open profile'));
    await tester.pumpAndSettle();

    expect(find.text(creator.displayName), findsOneWidget);
  });
}
