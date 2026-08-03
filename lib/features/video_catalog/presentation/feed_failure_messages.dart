import 'package:ghostr/core/errors/boundary_failure.dart';

/// Viewer-facing messages for unexpected feed errors, translated and
/// reported through the boundary-failure pipeline.
String unexpectedFeedLoadMessage(Object error, StackTrace stackTrace) {
  return translatedBoundaryFailure(
    source: 'FeedCubit.load',
    message: 'Could not load the Nostr video feed.',
    error: error,
    stackTrace: stackTrace,
  ).message;
}

String unexpectedFeedLikeMessage(Object error, StackTrace stackTrace) {
  return translatedBoundaryFailure(
    source: 'FeedCubit.toggleLike',
    message: 'Could not update this like.',
    error: error,
    stackTrace: stackTrace,
  ).message;
}

String unexpectedFeedBlockMessage(Object error, StackTrace stackTrace) {
  return translatedBoundaryFailure(
    source: 'FeedCubit.blockCreator',
    message: 'Could not block this creator.',
    error: error,
    stackTrace: stackTrace,
  ).message;
}
