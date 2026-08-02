import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('publishes every APK variant through the reusable publisher', () {
    final workflow = File(
      '.github/workflows/release_android.yaml',
    ).readAsStringSync();

    expect(
      workflow,
      allOf(
        contains('name: Check out release tooling'),
        contains('name: Prepare uniquely named APK assets'),
        contains(r'ghostr-${GITHUB_REF_NAME}-arm64-v8a.apk'),
        contains(r'ghostr-${GITHUB_REF_NAME}-armeabi-v7a.apk'),
        contains(r'ghostr-${GITHUB_REF_NAME}-x86_64.apk'),
        contains('sh tool/publish_android_release.sh'),
        allOf(
          isNot(contains('actions/create-release@v1')),
          isNot(contains('actions/upload-release-asset@v1')),
        ),
      ),
    );
  });
}
