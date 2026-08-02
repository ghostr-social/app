import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('coverage gate rejects an omitted executable source file', () {
    final fixture = Directory.systemTemp.createTempSync('ghostr-coverage-');
    final source = Directory('${fixture.path}/lib')..createSync();
    File('${source.path}/covered.dart').writeAsStringSync('void covered() {}');
    File('${source.path}/missing.dart').writeAsStringSync('void missing() {}');
    final lcov = File('${fixture.path}/lcov.info')
      ..writeAsStringSync('SF:lib/covered.dart\nDA:1,1\nend_of_record\n');
    final exclusions = File('${fixture.path}/exclusions.txt')..createSync();

    final result = Process.runSync('sh', [
      'tool/check_dart_coverage_sources.sh',
      lcov.path,
      source.path,
      exclusions.path,
    ]);
    fixture.deleteSync(recursive: true);

    expect(result.exitCode, isNonZero);
    expect('${result.stdout}${result.stderr}', contains('missing.dart'));
  });
}
