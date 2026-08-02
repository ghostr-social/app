import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_profile_repository.dart';

import '../support/profile_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('announces the creator-profile loading state', (tester) async {
    await tester.pumpWidget(profileScreenHarness(
      profile: _PendingProfileRepository(),
      viewer: sampleSession().profile,
      profileId: sampleCreator().id,
    ));
    await tester.pump();

    expect(find.bySemanticsLabel('Loading creator profile'), findsOneWidget);
    expect(find.byType(CircularProgressIndicator), findsOneWidget);
  });
}

class _PendingProfileRepository implements VideoProfileRepository {
  final _load = Completer<ProfileDetails>();

  @override
  Future<ProfileDetails> loadProfile(
    ProfileSummary viewer,
    ProfileId profileId,
  ) =>
      _load.future;

  @override
  Future<bool> toggleBlock(ProfileId profileId) async => false;

  @override
  Future<bool> toggleFollow(ProfileId profileId) async => false;
}
