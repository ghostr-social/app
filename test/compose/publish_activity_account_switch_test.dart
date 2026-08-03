import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/activity/data/local_activity_repository.dart';
import 'package:ghostr/features/activity/data/nostr_activity_repository.dart';
import 'package:ghostr/features/compose/domain/publish_video_workflow.dart';
import 'package:ghostr/features/publish/domain/video_publication.dart';
import 'package:ghostr/features/publish/domain/video_publishing_repository.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';
import '../support/sample_data.dart';

void main() {
  test('records a completed publish for the account that started it', () async {
    SharedPreferences.setMockInitialValues({});
    var account = NostrPublicKeyHex.parse(testViewerPublicKey);
    final local = LocalActivityRepository(
      await SharedPreferences.getInstance(),
      accountScope: AccountStorageScope(() => account),
    );
    final publishing = _DelayedPublishingRepository();
    final workflow = DefaultPublishVideoWorkflow(
      publishing: publishing,
      activity: NostrActivityRepository(
        client: FakeNostrEventClient(publicKeyHex: testViewerPublicKey),
        local: local,
        failureReporter: RecordingFailureReporter(),
      ),
      clock: () => DateTime.utc(2026, 8, 2),
      failureReporter: RecordingFailureReporter(),
    );

    final pending = workflow.publish(
      session: sampleSession(),
      media: sampleMedia(),
      rawCaption: 'Caption',
    );
    await publishing.started.future;
    account = NostrPublicKeyHex.parse(testAuthorPublicKey);
    publishing.release.complete();
    await pending;

    expect(await local.load(), isEmpty);
    account = NostrPublicKeyHex.parse(testViewerPublicKey);
    expect(await local.load(), hasLength(1));
  });
}

class _DelayedPublishingRepository implements VideoPublishingRepository {
  final started = Completer<void>();
  final release = Completer<void>();

  @override
  Future<VideoPublication> publish({
    required UserSession session,
    required SelectedMedia media,
    required String caption,
  }) async {
    started.complete();
    await release.future;
    return VideoPublication.stored(samplePost(caption: caption));
  }
}
