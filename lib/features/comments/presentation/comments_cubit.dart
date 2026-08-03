import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/comments/domain/video_comments_repository.dart';
import 'package:ghostr/features/comments/presentation/comments_state.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

export 'comments_state.dart';

class CommentsCubit extends DisposalSafeCubit<CommentsState> {
  CommentsCubit(this._repository, this._post)
      : super(const CommentsState.loading());

  final VideoCommentsRepository _repository;
  final VideoPost _post;

  Future<void> load() async {
    emit(const CommentsState.loading());
    try {
      emit(CommentsState.content(await _repository.loadComments(_post)));
    } on AppFailure catch (failure) {
      emit(CommentsState.failure(failure.message));
    } on Object catch (error, stackTrace) {
      emit(CommentsState.failure(_unexpectedLoad(error, stackTrace)));
    }
  }

  void selectReply(VideoComment comment) {
    if (_hasContent && !state.isPosting) emit(state.withReply(comment));
  }

  Future<bool> publish(String rawContent) async {
    final content = rawContent.trim();
    if (!_canPublish(content)) return false;
    emit(state.posting());
    try {
      final comment = await _publish(content);
      emit(state.published(comment));
      return true;
    } on AppFailure catch (failure) {
      return _rejectPublish(failure.message);
    } on Object catch (error, stackTrace) {
      return _rejectPublish(_unexpectedPublish(error, stackTrace));
    }
  }

  void clearNotice() {
    if (state.notice != null) emit(state.withoutNotice());
  }

  bool get _hasContent {
    return state is CommentsContent;
  }

  bool _canPublish(String content) {
    return content.isNotEmpty && _hasContent && !state.isPosting;
  }

  Future<VideoComment> _publish(String content) {
    return _repository.publishComment(
      post: _post,
      content: content,
      replyTo: state.replyTo,
    );
  }

  bool _rejectPublish(String message) {
    emit(state.withNotice(message));
    return false;
  }

  String _unexpectedLoad(Object error, StackTrace stackTrace) {
    return translatedBoundaryFailure(
      source: 'CommentsCubit.load',
      message: 'Could not load comments from relays.',
      error: error,
      stackTrace: stackTrace,
    ).message;
  }

  String _unexpectedPublish(Object error, StackTrace stackTrace) {
    return translatedBoundaryFailure(
      source: 'CommentsCubit.publish',
      message: 'Could not publish this comment.',
      error: error,
      stackTrace: stackTrace,
    ).message;
  }
}
