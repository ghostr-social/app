import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/activity/data/local_activity_repository.dart';
import 'package:ghostr/features/activity/data/nostr_activity_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/toggle_profile_follow_workflow.dart';
import 'package:ghostr/features/video_catalog/domain/video_profile_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';
import '../support/sample_data.dart';

void main() {
  test('records a completed follow for the account that started it', () async {
    SharedPreferences.setMockInitialValues({});
    var account = NostrPublicKeyHex.parse(testViewerPublicKey);
    final local = LocalActivityRepository(
      await SharedPreferences.getInstance(),
      accountScope: AccountStorageScope(() => account),
    );
    final profile = _DelayedProfileRepository();
    final workflow = DefaultToggleProfileFollowWorkflow(
      profile: profile,
      activity: NostrActivityRepository(
        client: FakeNostrEventClient(publicKeyHex: testViewerPublicKey),
        local: local,
        failureReporter: RecordingFailureReporter(),
      ),
      clock: () => DateTime.utc(2026, 8, 2),
      failureReporter: RecordingFailureReporter(),
    );

    final pending = workflow.toggle(sampleProfileDetails());
    await profile.started.future;
    account = NostrPublicKeyHex.parse(testAuthorPublicKey);
    profile.release.complete();
    await pending;

    expect(await local.load(), isEmpty);
    account = NostrPublicKeyHex.parse(testViewerPublicKey);
    expect(await local.load(), hasLength(1));
  });
}

class _DelayedProfileRepository implements VideoProfileRepository {
  final started = Completer<void>();
  final release = Completer<void>();

  @override
  Future<bool> toggleFollow(ProfileId profileId) async {
    started.complete();
    await release.future;
    return true;
  }

  @override
  Future<ProfileDetails> loadProfile(
          ProfileSummary viewer, ProfileId profileId) =>
      throw UnimplementedError();

  @override
  Future<bool> toggleBlock(ProfileId profileId) => throw UnimplementedError();
}
