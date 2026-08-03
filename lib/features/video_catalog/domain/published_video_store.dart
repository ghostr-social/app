import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

abstract interface class PublishedVideoStore {
  PublishedVideoStore snapshotForActiveAccount();

  NostrPublicKeyHex get accountPublicKey;

  Future<List<VideoPost>> loadPublishedPosts();

  Future<void> savePublishedPosts(List<VideoPost> posts);
}
