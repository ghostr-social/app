import 'package:ghostr/features/comments/domain/video_comment.dart';

enum CommentsStatus { loading, empty, ready, failure }

class CommentsState {
  const CommentsState._({
    required this.status,
    this.comments = const [],
    this.replyTo,
    this.isPosting = false,
    this.message,
    this.notice,
  });

  const CommentsState.loading() : this._(status: CommentsStatus.loading);

  const CommentsState.failure(String message)
      : this._(status: CommentsStatus.failure, message: message);

  factory CommentsState.content(List<VideoComment> comments) {
    final status =
        comments.isEmpty ? CommentsStatus.empty : CommentsStatus.ready;
    return CommentsState._(
      status: status,
      comments: List<VideoComment>.unmodifiable(comments),
    );
  }

  final CommentsStatus status;
  final List<VideoComment> comments;
  final VideoComment? replyTo;
  final bool isPosting;
  final String? message;
  final String? notice;

  CommentsState withReply(VideoComment comment) {
    return CommentsState._(
      status: status,
      comments: comments,
      replyTo: comment,
      isPosting: isPosting,
      message: message,
      notice: notice,
    );
  }

  CommentsState posting() {
    return CommentsState._(
      status: status,
      comments: comments,
      replyTo: replyTo,
      isPosting: true,
      message: message,
      notice: notice,
    );
  }

  CommentsState published(VideoComment comment) {
    return CommentsState.content([...comments, comment]);
  }

  CommentsState withNotice(String value) {
    return CommentsState._(
      status: status,
      comments: comments,
      replyTo: replyTo,
      isPosting: isPosting,
      message: message,
      notice: value,
    );
  }

  CommentsState withoutNotice() {
    return CommentsState._(
      status: status,
      comments: comments,
      replyTo: replyTo,
      isPosting: isPosting,
      message: message,
    );
  }
}
