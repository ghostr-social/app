import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/app/profile_route_request.dart';
import 'package:ghostr/app/router/app_router.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_screen.dart';

import '../support/fakes.dart';
import '../support/recording_video_playback_port.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('a profile feed route starts on the tapped video', (
    tester,
  ) async {
    final creator = sampleCreator();
    final first = samplePost(
      id: 'clip-1',
      caption: 'First clip',
      creator: creator,
    );
    final second = samplePost(
      id: 'clip-2',
      caption: 'Second clip',
      creator: creator,
    );
    final playback = RecordingVideoPlaybackPort();
    final controllers = AppControllerFactory(
      buildFakeDependencies(
        session: sampleSession(),
        catalogRepository: FakeVideoCatalogRepository(
          forYouFeed: [samplePost(id: 'other', caption: 'Unrelated clip')],
          feed: FakeFeedScenario(
            profiles: {
              creator.id: sampleProfileDetails(
                profile: creator,
                posts: [first, second],
              ),
            },
          ),
        ),
        device: FakeDeviceDependencies(playback: playback),
      ),
    );
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) => ElevatedButton(
            onPressed: () => Navigator.of(context).push(
              AppRouter.profileFeed(
                ProfileFeedRouteRequest(
                  session: sampleSession(),
                  post: second,
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

    expect(find.widgetWithText(AppBar, creator.displayName), findsOneWidget);
    expect(playback.activity[second.media.debugLabel]!.last, isTrue);
    expect(playback.activity[first.media.debugLabel]?.last, isNot(isTrue));

    await tester.tap(find.byTooltip('Open profile'));
    await tester.pumpAndSettle();
    expect(find.byType(ProfileScreen), findsOneWidget);
    expect(playback.activity[second.media.debugLabel]!.last, isFalse);
    tester.state<NavigatorState>(find.byType(Navigator)).pop();
    await tester.pumpAndSettle();
    expect(playback.activity[second.media.debugLabel]!.last, isTrue);
  });
}
