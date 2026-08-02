import 'package:ghostr/src/rust/video/video.dart';

import 'nostr_test_values.dart';

FfiVideoDownload ffiVideo({
  required String id,
  required FfiUserData user,
  FfiVideoFixtureOptions options = const FfiVideoFixtureOptions(),
  FfiNostrEventIdentity? event,
}) {
  final mediaUrl = options.mediaUrl ?? 'https://source.example/$id.mp4';
  return FfiVideoDownload(
    id: id,
    url: mediaUrl,
    localPath: options.localPath,
    event: event ?? ffiNostrEvent(),
    nostr: FfiNostrVideo(
      id: id,
      user: user,
      title: options.title ?? 'Video $id',
      songName: options.songName,
      likes: options.likes,
      comments: '2',
      url: mediaUrl,
      delivery: options.delivery,
    ),
  );
}

FfiNostrEventIdentity ffiNostrEvent({
  String eventId = testEventId,
  String authorPublicKeyHex = testCreatorPublicKey,
  String? identifier = 'dance',
}) {
  return FfiNostrEventIdentity(
    eventId: eventId,
    authorPublicKeyHex: authorPublicKeyHex,
    kind: BigInt.from(34235),
    identifier: identifier,
    createdAt: BigInt.from(1785628800),
    content: 'Relay dance',
  );
}

class FfiVideoFixtureOptions {
  const FfiVideoFixtureOptions({
    this.localPath,
    this.likes = '4',
    this.title,
    this.songName = 'Original sound',
    this.mediaUrl,
    this.delivery = FfiVideoDelivery.progressive,
  });

  final String? localPath;
  final String likes;
  final String? title;
  final String songName;
  final String? mediaUrl;
  final FfiVideoDelivery delivery;
}
