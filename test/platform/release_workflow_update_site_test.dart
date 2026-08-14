import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('a green push publishes stable metadata with the release', () {
    final workflow = File(
      '.github/workflows/release_android.yaml',
    ).readAsStringSync();
    final release = workflow.indexOf('sh tool/publish_android_release.sh');
    final syncJob = workflow.indexOf('sync-update-site:');
    final credential = workflow.indexOf('UPDATE_SITE_TOKEN');
    final sync = workflow.indexOf('sh tool/sync_android_update_site.sh');

    expect(
      workflow,
      allOf(
        allOf(
          contains('name: Verify Flutter quality gates'),
          contains('flutter analyze'),
          contains('make test-coverage'),
          contains('make coverage-summary'),
        ),
        allOf(
          contains('sh tool/generate_android_update_manifest.sh'),
          contains('release-assets/stable.json'),
          contains('UPDATE_SITE_TOKEN'),
          contains('needs: [version, publish]'),
          contains('timeout-minutes: 20'),
          contains('gh release download "\$RELEASE_TAG"'),
        ),
        contains(
          'sh tool/sync_android_update_site.sh '
          '"\$RELEASE_TAG" release-assets/stable.json',
        ),
        isNot(contains('--clobber')),
      ),
    );
    expect(release, greaterThanOrEqualTo(0));
    expect(syncJob, greaterThan(release));
    expect(credential, greaterThan(syncJob));
    expect(sync, greaterThan(syncJob));
  });
}
