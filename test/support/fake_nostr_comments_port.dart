import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/comments/domain/nostr_comments_port.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

import 'nostr_test_values.dart';

class FakeNostrCommentsPort implements NostrCommentsPort {
  final List<VideoComment> comments = <VideoComment>[];
  AppFailure? loadFailure;

  @override
  Future<List<VideoComment>> load(NostrEventReference reference) async {
    if (loadFailure case final failure?) throw failure;
    return <VideoComment>[...comments];
  }

  @override
  Future<VideoComment> publish({
    required NostrEventReference reference,
    required String content,
    VideoComment? replyTo,
  }) async {
    final comment = VideoComment(
      identity: VideoCommentIdentity.parse(
        id: publishedEventId(comments.length + 1),
        authorPublicKeyHex: testViewerPublicKey,
      ),
      text: VideoCommentText(authorLabel: 'You', content: content),
      createdAt: DateTime(2026, 8, 2),
      parentCommentId: replyTo?.id,
    );
    comments.add(comment);
    return comment;
  }
}
