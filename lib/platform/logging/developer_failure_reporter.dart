import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/errors/failure_reporter.dart';

typedef FailureLogger = void Function({
  required String source,
  required String message,
  required Object error,
  required StackTrace stackTrace,
});

class DeveloperFailureReporter implements FailureReporter {
  const DeveloperFailureReporter({FailureLogger logger = logBoundaryFailure})
      : _logger = logger;

  final FailureLogger _logger;

  @override
  void report({
    required String source,
    required Object error,
    required StackTrace stackTrace,
  }) {
    _logger(
      source: source,
      message: _message(error),
      error: error,
      stackTrace: stackTrace,
    );
  }

  String _message(Object error) {
    return error is AppFailure
        ? error.message
        : 'A recoverable operation failed.';
  }
}
