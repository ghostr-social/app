import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/publish/data/nostr_video_publisher.dart';
import 'package:ghostr/features/publish/domain/uploaded_video_media.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/fake_video_media_upload_port.dart';
import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';

void main() {
  test('a completed publish remains attributed to its initiating account',
      () async {
    final barrier = Completer<void>();
    final client = _DelayedPublishClient(barrier);
    final publisher = NostrVideoPublisher(
      eventClient: client,
      mediaUploader: FakeVideoMediaUploadPort(_uploadedMedia()),
    );

    final pending = publisher.publish(
      session: sampleSession(),
      media: sampleMedia(),
      caption: 'Account A post',
    );
    await Future<void>.delayed(Duration.zero);
    client.publicKeyHex = NostrPublicKeyHex.parse(testAuthorPublicKey);
    barrier.complete();

    final post = await pending;
    expect(post.nostrReference?.authorPublicKeyHex, testViewerPublicKey);
    expect(client.events.single.authorPublicKeyHex, testViewerPublicKey);
    expect(
      client.expectedAuthor,
      NostrPublicKeyHex.parse(testViewerPublicKey),
    );
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

class _DelayedPublishClient extends FakeNostrEventClient {
  _DelayedPublishClient(this.barrier)
      : super(publicKeyHex: testViewerPublicKey);

  final Completer<void> barrier;
  NostrPublicKeyHex? expectedAuthor;

  @override
  Future<NostrEventId> publish(
    NostrUnsignedEvent event, {
    required NostrPublicKeyHex expectedAuthor,
  }) async {
    this.expectedAuthor = expectedAuthor;
    await barrier.future;
    final id = NostrEventId.parse(publishedEventId(events.length + 1));
    events.add(event.toRecord(
      id: id,
      authorPublicKeyHex: expectedAuthor,
      createdAt: 1700000000,
    ));
    return id;
  }
}
