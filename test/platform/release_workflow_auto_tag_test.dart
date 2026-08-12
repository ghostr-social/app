import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('publishes a freshly tagged release for every green main push', () {
    final workflow = File(
      '.github/workflows/release_android.yaml',
    ).readAsStringSync();

    expect(
      workflow,
      allOf(
        contains('sh tool/next_release_tag.sh'),
        contains(r'RELEASE_TAG: ${{ needs.version.outputs.tag }}'),
        contains("github.ref == 'refs/heads/main'"),
        contains(r'"$GITHUB_SHA"'),
        contains("&& 'android-release-publisher'"),
        contains('cancel-in-progress: false'),
      ),
    );
  });
}
