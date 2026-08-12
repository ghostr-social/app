import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test(
    'pull requests cannot receive release signing secrets or write access',
    () {
      final workflow = File(
        '.github/workflows/release_android.yaml',
      ).readAsStringSync();
      final guardedSigningStep = RegExp(
        r'- name: Set up release signing\n'
        r"        if: github\.event_name == 'push'\n"
        r'        env:\n'
        r'          ANDROID_KEYSTORE_BASE64:',
      );
      final signingSecret = RegExp(
        r'ANDROID_KEYSTORE_BASE64: \$\{\{ secrets\.ANDROID_KEYSTORE_BASE64 \}\}',
      );

      expect(workflow, contains('permissions:\n  contents: read'));
      expect(guardedSigningStep.allMatches(workflow), hasLength(3));
      expect(signingSecret.allMatches(workflow), hasLength(3));
      expect(
        workflow,
        contains(
          '  publish:\n    name: Publish Release\n'
          '    permissions:\n      contents: write',
        ),
      );
    },
  );
}
