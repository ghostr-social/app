import 'package:ghostr/core/errors/boundary_failure.dart';

String unexpectedProfileLoadFailure(Object error, StackTrace stackTrace) {
  return _unexpectedProfileFailure(
    'ProfileCubit.load',
    'Could not load this profile.',
    error,
    stackTrace,
  );
}

String unexpectedProfileFollowFailure(Object error, StackTrace stackTrace) {
  return _unexpectedProfileFailure(
    'ProfileCubit.toggleFollow',
    'Could not update this follow.',
    error,
    stackTrace,
  );
}

String unexpectedProfileBlockFailure(Object error, StackTrace stackTrace) {
  return _unexpectedProfileFailure(
    'ProfileCubit.toggleBlock',
    'Could not update this block.',
    error,
    stackTrace,
  );
}

String unexpectedProfileMetadataFailure(Object error, StackTrace stackTrace) {
  return _unexpectedProfileFailure(
    'ProfileCubit.refreshMetadata',
    'Could not refresh profile details.',
    error,
    stackTrace,
  );
}

String _unexpectedProfileFailure(
  String source,
  String message,
  Object error,
  StackTrace stackTrace,
) {
  return translatedBoundaryFailure(
    source: source,
    message: message,
    error: error,
    stackTrace: stackTrace,
  ).message;
}
