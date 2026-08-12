/// How `apksigner verify --print-certs` labels the signing certificate line:
/// build-tools below 35 use `Signer #1 ...`, newer releases `V2 Signer: ...`.
enum ApksignerCertificateFormat {
  legacy('Signer #1 certificate SHA-256 digest'),
  modern('V2 Signer: certificate SHA-256 digest');

  const ApksignerCertificateFormat(this.label);

  final String label;
}

final class AndroidUpdateApkFixture {
  const AndroidUpdateApkFixture({
    this.packageName = 'app.ghostr',
    this.versionName = '1.2.3',
    this.versionCode = '1002003',
    this.certificate = stableCertificate,
    this.abi = 'arm64-v8a',
    this.includesIntegrationTest = false,
    this.apksignerFormat = ApksignerCertificateFormat.legacy,
  });

  final String packageName;
  final String versionName;
  final String versionCode;
  final String certificate;
  final String abi;
  final bool includesIntegrationTest;
  final ApksignerCertificateFormat apksignerFormat;
}

const stableCertificate =
    '1e2c0712ebbc909cb2aa7ea9af97ae620639f1e01463f28f6ee1e68c1ed3b340';
