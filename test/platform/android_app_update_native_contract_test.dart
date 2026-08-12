import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('Android owns a hardened versioned self-update bridge', () {
    final manifest = File(
      'android/app/src/main/AndroidManifest.xml',
    ).readAsStringSync();
    final build = File('android/app/build.gradle').readAsStringSync();
    final networkSecurity = File(
      'android/app/src/main/res/xml/network_security_config.xml',
    ).readAsStringSync();
    final kotlin = Directory('android/app/src/main/kotlin/social/ghostr')
        .listSync(recursive: true)
        .whereType<File>()
        .map((file) => file.readAsStringSync())
        .join('\n');

    expect(build, contains('namespace = "social.ghostr"'));
    expect(build, contains('applicationId = "app.ghostr"'));
    for (final permission in [
      'android.permission.REQUEST_INSTALL_PACKAGES',
      'android.permission.UPDATE_PACKAGES_WITHOUT_USER_ACTION',
      'android.permission.ACCESS_NETWORK_STATE',
    ]) {
      expect(manifest, contains(permission));
    }
    expect(manifest, contains('android:usesCleartextTraffic="false"'));
    expect(
      manifest,
      contains('android:networkSecurityConfig="@xml/network_security_config"'),
    );
    expect(networkSecurity, contains('cleartextTrafficPermitted="false"'));
    expect(
      networkSecurity,
      contains('<domain includeSubdomains="false">127.0.0.1</domain>'),
    );
    expect(
      networkSecurity,
      isNot(contains('<base-config cleartextTrafficPermitted="true"')),
    );
    expect(kotlin, contains('social.ghostr/app_update/v1'));
    expect(kotlin, contains('getPackageArchiveInfo'));
    expect(kotlin, contains('expectedVersionCode'));
    expect(kotlin, contains('apkContentsSigners'));
    expect(kotlin, contains('signingCertificateHistory'));
    expect(kotlin, contains('USER_ACTION_NOT_REQUIRED'));
    expect(kotlin, contains('STATUS_PENDING_USER_ACTION'));
    expect(kotlin, contains('InstallStatusReceiver::class.java'));
    expect(manifest, contains('android:name=".InstallStatusReceiver"'));
    expect(manifest, contains('android:exported="false"'));
  });
}
