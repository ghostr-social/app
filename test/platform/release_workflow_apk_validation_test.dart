import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('validates every release APK before generating stable metadata', () {
    final workflow = File(
      '.github/workflows/release_android.yaml',
    ).readAsStringSync();
    final validation = 'sh tool/check_android_update_apk.sh';
    final validationIndex = workflow.indexOf(validation);
    final manifestIndex = workflow.indexOf(
      'sh tool/generate_android_update_manifest.sh',
    );

    expect(workflow, contains('sdkmanager "build-tools;36.0.0"'));
    expect(validation.allMatches(workflow), hasLength(3));
    expect(workflow, contains(r'"$BUILD_NAME" "$BUILD_NUMBER"'));
    expect(validationIndex, lessThan(manifestIndex));
  });
}
