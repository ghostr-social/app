import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/profile_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('disables profile social actions while a write is pending',
      (tester) async {
    final creator = sampleCreator();
    final repository = _PendingSocialRepository(
      forYouFeed: [samplePost(creator: creator)],
      feed: FakeFeedScenario(profiles: {
        creator.id: sampleProfileDetails(profile: creator),
      }),
    );
    await tester.pumpWidget(profileScreenHarness(
      profile: repository,
      viewer: sampleSession().profile,
      profileId: creator.id,
    ));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Follow'));
    await tester.pump();

    expect(tester.widget<FilledButton>(find.byType(FilledButton)).onPressed,
        isNull);
    expect(tester.widget<OutlinedButton>(find.byType(OutlinedButton)).onPressed,
        isNull);
  });
}

class _PendingSocialRepository extends FakeVideoCatalogRepository {
  _PendingSocialRepository({required super.forYouFeed, required super.feed});

  @override
  Future<bool> toggleFollow(String profileId) => Completer<bool>().future;
}
