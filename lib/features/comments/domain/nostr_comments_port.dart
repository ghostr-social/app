import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

abstract interface class NostrCommentsPort {
  Future<List<VideoComment>> load(NostrEventReference reference);

  Future<Map<NostrEventId, List<VideoComment>>> loadBatch(
    List<NostrEventReference> references,
  );

  Future<VideoComment> publish({
    required NostrEventReference reference,
    required String content,
    VideoComment? replyTo,
  });
}
