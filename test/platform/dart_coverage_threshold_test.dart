import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('Dart coverage gate accepts 80 percent but rejects less', () {
    final fixture = Directory.systemTemp.createTempSync('ghostr-coverage-');
    final accepted = _writeCoverage(fixture, 'accepted.info', 8);
    final rejected = _writeCoverage(fixture, 'rejected.info', 7);

    final acceptedResult = _checkCoverage(accepted);
    final rejectedResult = _checkCoverage(rejected);
    fixture.deleteSync(recursive: true);

    expect(acceptedResult.exitCode, 0);
    expect(rejectedResult.exitCode, isNonZero);
    expect('${rejectedResult.stdout}', contains('below 80%'));
  });
}

File _writeCoverage(Directory fixture, String name, int coveredLines) {
  final lines = <String>[
    'SF:lib/core/example.dart',
    for (var line = 1; line <= 10; line++)
      'DA:$line,${line <= coveredLines ? 1 : 0}',
    'end_of_record',
  ];
  return File('${fixture.path}/$name')..writeAsStringSync(lines.join('\n'));
}

ProcessResult _checkCoverage(File coverage) {
  return Process.runSync('awk', [
    '-f',
    'tool/check_dart_coverage.awk',
    coverage.path,
  ]);
}
