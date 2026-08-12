final class AndroidUpdateApkFixture {
  const AndroidUpdateApkFixture({
    this.packageName = 'app.ghostr',
    this.versionName = '1.2.3',
    this.versionCode = '1002003',
    this.certificate = stableCertificate,
    this.abi = 'arm64-v8a',
    this.includesIntegrationTest = false,
  });

  final String packageName;
  final String versionName;
  final String versionCode;
  final String certificate;
  final String abi;
  final bool includesIntegrationTest;
}

const stableCertificate =
    '1e2c0712ebbc909cb2aa7ea9af97ae620639f1e01463f28f6ee1e68c1ed3b340';
