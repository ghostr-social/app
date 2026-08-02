import 'dart:async';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';

Future<T> guardPreferenceStorage<T>(
  String failureMessage,
  FutureOr<T> Function() operation,
) async {
  try {
    return await operation();
  } on Object catch (error, stackTrace) {
    throw translatedBoundaryFailure(
      source: 'ghostr.storage.preferences',
      message: failureMessage,
      error: error,
      stackTrace: stackTrace,
    );
  }
}

Future<void> requirePreferenceWrite(
  String failureMessage,
  Future<bool> Function() write,
) async {
  final didWrite = await guardPreferenceStorage(failureMessage, write);
  if (!didWrite) throw AppFailure(failureMessage);
}
