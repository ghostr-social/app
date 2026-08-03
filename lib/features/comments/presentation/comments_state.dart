import 'package:ghostr/features/comments/domain/video_comment.dart';

enum CommentsStatus { loading, empty, ready, failure }

sealed class CommentsState {
  const CommentsState({this.notice});

  const factory CommentsState.loading() = CommentsLoading;

  const factory CommentsState.failure(String message) = CommentsFailure;

  factory CommentsState.content(List<VideoComment> comments) {
    return CommentsContent(List<VideoComment>.unmodifiable(comments));
  }

  final String? notice;
  CommentsStatus get status;
  List<VideoComment> get comments => const <VideoComment>[];
  VideoComment? get replyTo => null;
  bool get isPosting => false;
  String? get message => null;

  CommentsState withReply(VideoComment comment) => this;

  CommentsState posting() => this;

  CommentsState published(VideoComment comment) => this;

  CommentsState withNotice(String value) => this;

  CommentsState withoutNotice() => this;
}

final class CommentsLoading extends CommentsState {
  const CommentsLoading();

  @override
  CommentsStatus get status => CommentsStatus.loading;
}

final class CommentsFailure extends CommentsState {
  const CommentsFailure(this.failureMessage, {super.notice});

  final String failureMessage;

  @override
  CommentsStatus get status => CommentsStatus.failure;

  @override
  String get message => failureMessage;

  @override
  CommentsState withNotice(String value) {
    return CommentsFailure(failureMessage, notice: value);
  }
}

final class CommentsContent extends CommentsState {
  CommentsContent(
    List<VideoComment> comments, {
    this.replyTo,
    this.isPosting = false,
    super.notice,
  }) : comments = List<VideoComment>.unmodifiable(comments);

  @override
  final List<VideoComment> comments;
  @override
  final VideoComment? replyTo;
  @override
  final bool isPosting;

  @override
  CommentsStatus get status =>
      comments.isEmpty ? CommentsStatus.empty : CommentsStatus.ready;

  @override
  CommentsState withReply(VideoComment comment) {
    return _copy(replyTo: comment);
  }

  @override
  CommentsState posting() => _copy(isPosting: true);

  @override
  CommentsState published(VideoComment comment) {
    return CommentsContent(<VideoComment>[...comments, comment]);
  }

  @override
  CommentsState withNotice(String value) {
    return _copy(isPosting: false, notice: value);
  }

  @override
  CommentsState withoutNotice() => _copy(clearNotice: true);

  CommentsContent _copy({
    VideoComment? replyTo,
    bool? isPosting,
    String? notice,
    bool clearNotice = false,
  }) {
    return CommentsContent(
      comments,
      replyTo: replyTo ?? this.replyTo,
      isPosting: isPosting ?? this.isPosting,
      notice: clearNotice ? null : notice ?? this.notice,
    );
  }
}
