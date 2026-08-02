import 'dart:developer';

import 'package:ghostr/core/errors/app_failure.dart';

AppFailure translatedBoundaryFailure({
  required String source,
  required String message,
  required Object error,
  required StackTrace stackTrace,
}) {
  logBoundaryFailure(
    source: source,
    message: message,
    error: error,
    stackTrace: stackTrace,
  );
  return AppFailure(message);
}

void logBoundaryFailure({
  required String source,
  required String message,
  required Object error,
  required StackTrace stackTrace,
}) {
  log(
    message,
    name: source,
    error: error,
    stackTrace: stackTrace,
  );
}
