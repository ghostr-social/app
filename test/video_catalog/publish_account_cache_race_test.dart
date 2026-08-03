import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/video_catalog/data/hybrid_video_publishing_repository.dart';
import 'package:ghostr/features/video_catalog/data/local_video_store.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_video_publisher_port.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';
import '../support/sample_data.dart';

void main() {
  test('a completed publish caches only for its initiating account', () async {
    SharedPreferences.setMockInitialValues({});
    var account = NostrPublicKeyHex.parse(testViewerPublicKey);
    final local = LocalVideoStore(
      await SharedPreferences.getInstance(),
      accountScope: AccountStorageScope(() => account),
    );
    final remote = _DelayedPublisher();
    final repository = HybridVideoPublishingRepository(
      local,
      remote,
      RecordingFailureReporter(),
    );

    final pending = repository.publish(
      session: sampleSession(),
      media: sampleMedia(),
      caption: 'Pinned cache',
    );
    await remote.started.future;
    account = NostrPublicKeyHex.parse(testAuthorPublicKey);
    remote.release.complete();
    await pending;

    expect(await local.loadPublishedPosts(), isEmpty);
    account = NostrPublicKeyHex.parse(testViewerPublicKey);
    expect((await local.loadPublishedPosts()).single.caption, 'Pinned cache');
  });
}

class _DelayedPublisher extends FakeNostrVideoPublisherPort {
  final started = Completer<void>();
  final release = Completer<void>();

  @override
  Future<VideoPost> publish({
    required UserSession session,
    required SelectedMedia media,
    required String caption,
  }) async {
    started.complete();
    await release.future;
    return super.publish(session: session, media: media, caption: caption);
  }
}
