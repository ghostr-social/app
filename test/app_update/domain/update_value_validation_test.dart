import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/domain/update_package_sha256.dart';

void main() {
  test('validates updater values at their construction boundary', () {
    expect(AndroidAbi.tryParse('arm64-v8a'), AndroidAbi.arm64V8a);
    expect(AndroidAbi.tryParse('x86'), isNull);
    expect(() => AndroidVersionCode(0), throwsArgumentError);
    expect(() => AndroidVersionCode(2100000001), throwsArgumentError);
    expect(AndroidVersionCode(1).compareTo(AndroidVersionCode(2)), lessThan(0));
    expect(AndroidVersionCode(2), AndroidVersionCode(2));
    expect(AndroidVersionCode(2).hashCode, 2.hashCode);
    expect(UpdatePackageSha256.tryParse('a' * 64)?.value, 'a' * 64);
    expect(UpdatePackageSha256.tryParse('A' * 64), isNull);
    expect(() => UpdatePackageSha256.parse('short'), throwsFormatException);
  });
}
