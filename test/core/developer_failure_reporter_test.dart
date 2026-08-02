import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/logging/developer_failure_reporter.dart';

void main() {
  test('reports safe and unexpected recoverable failures', () {
    final messages = <String>[];
    final sources = <String>[];
    final reporter = DeveloperFailureReporter(
      logger: ({
        required source,
        required message,
        required error,
        required stackTrace,
      }) {
        sources.add(source);
        messages.add(message);
      },
    );

    reporter.report(
      source: 'ghostr.safe',
      error: const AppFailure('Relay read failed.'),
      stackTrace: StackTrace.empty,
    );
    reporter.report(
      source: 'ghostr.unexpected',
      error: StateError('socket closed'),
      stackTrace: StackTrace.empty,
    );

    expect(sources, ['ghostr.safe', 'ghostr.unexpected']);
    expect(messages, ['Relay read failed.', 'A recoverable operation failed.']);
  });
}
