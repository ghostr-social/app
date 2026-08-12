import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('a green push publishes stable metadata with the release', () {
    final workflow = File(
      '.github/workflows/release_android.yaml',
    ).readAsStringSync();

    expect(
      workflow,
      allOf(
        allOf(
          contains('name: Verify Flutter quality gates'),
          contains('flutter analyze'),
          contains('make test-coverage'),
          contains('make coverage-summary'),
        ),
        contains('sh tool/generate_android_update_manifest.sh'),
        contains('release-assets/stable.json'),
        isNot(contains('--clobber')),
      ),
    );
  });
}
