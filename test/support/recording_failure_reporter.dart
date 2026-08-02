import 'package:ghostr/core/errors/failure_reporter.dart';

class RecordingFailureReporter implements FailureReporter {
  final List<String> sources = <String>[];

  @override
  void report({
    required String source,
    required Object error,
    required StackTrace stackTrace,
  }) {
    sources.add(source);
  }
}
