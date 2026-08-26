import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('latest Axiom is a blocking completion gate', () {
    final agents = File('AGENTS.md').readAsStringSync();
    final makefile = File('Makefile').readAsStringSync();
    final workflow = File(
      '.github/workflows/release_android.yaml',
    ).readAsStringSync();
    final verifyStart = workflow.indexOf('  verify:');
    final buildStart = workflow.indexOf('  build-android-arm64:');
    final axiomCheck = workflow.indexOf('make axiom');

    expect(makefile, contains('AXIOM ?= axiom'));
    expect(
      makefile,
      contains(r'$(AXIOM) check --manifest-path rust/Cargo.toml'),
    );
    expect(workflow, contains('gu1p/axiom/main/install.sh'));
    expect(workflow, isNot(contains('AXIOM_VERSION')));
    expect(verifyStart, greaterThanOrEqualTo(0));
    expect(axiomCheck, greaterThan(verifyStart));
    expect(axiomCheck, lessThan(buildStart));
    expect(agents, contains('- `make axiom`'));
    expect(agents, contains('before declaring any task complete'));
  });
}
