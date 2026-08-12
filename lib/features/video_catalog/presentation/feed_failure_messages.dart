import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_operation_failure.dart';

/// Viewer-facing messages for unexpected feed errors, translated and
/// reported through the boundary-failure pipeline.
String feedLoadFailureMessage(FeedOperationFailure failure) {
  return _message(
    source: 'FeedCubit.load',
    message: 'Could not load the Nostr video feed.',
    failure: failure,
  );
}

String feedLikeFailureMessage(FeedOperationFailure failure) {
  return _message(
    source: 'FeedCubit.toggleLike',
    message: 'Could not update this like.',
    failure: failure,
  );
}

String feedBlockFailureMessage(FeedOperationFailure failure) {
  return _message(
    source: 'FeedCubit.blockCreator',
    message: 'Could not block this creator.',
    failure: failure,
  );
}

String feedFollowFailureMessage(FeedOperationFailure failure) {
  return _message(
    source: 'FeedCubit.followCreator',
    message: 'Could not follow this creator.',
    failure: failure,
  );
}

String _message({
  required String source,
  required String message,
  required FeedOperationFailure failure,
}) {
  final cause = failure.cause;
  if (cause is AppFailure) return cause.message;
  return translatedBoundaryFailure(
    source: source,
    message: message,
    error: cause,
    stackTrace: failure.stackTrace,
  ).message;
}
