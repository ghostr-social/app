import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/publish/data/nostr_video_publisher.dart';
import 'package:ghostr/features/publish/domain/uploaded_video_media.dart';
import 'package:ghostr/features/publish/domain/video_media_upload_port.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';

void main() {
  test('does not publish after the account changes during upload', () async {
    final barrier = Completer<UploadedVideoMedia>();
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final publisher = NostrVideoPublisher(
      eventClient: client,
      mediaUploader: _DelayedUploader(barrier.future),
    );

    final pending = publisher.publish(
      session: sampleSession(),
      media: sampleMedia(),
      caption: 'Account A post',
    );
    await Future<void>.delayed(Duration.zero);
    client.publicKeyHex = NostrPublicKeyHex.parse(testAuthorPublicKey);
    barrier.complete(_uploadedMedia());

    await expectLater(pending, throwsA(isA<AppFailure>()));
    expect(client.events, isEmpty);
  });
}

UploadedVideoMedia _uploadedMedia() {
  return UploadedVideoMedia(
    source: VideoMediaSource.remote('https://media.example/video.mp4'),
    metadata: VideoUploadMetadata(
      sha256: testEventId,
      mimeType: 'video/mp4',
      sizeBytes: 3,
    ),
  );
}

class _DelayedUploader implements VideoMediaUploadPort {
  const _DelayedUploader(this.result);

  final Future<UploadedVideoMedia> result;

  @override
  Future<UploadedVideoMedia> upload(SelectedMedia media) => result;
}
