import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/storage/secure_secret_store.dart';
import 'package:mocktail/mocktail.dart';

import '../support/mock_flutter_secure_storage.dart';

void main() {
  test('pending generated key uses a separate secure-storage slot', () async {
    final storage = MockFlutterSecureStorage();
    when(
      () => storage.write(
        key: 'ghostr.viewer.pendingSecret',
        value: any(named: 'value'),
      ),
    ).thenAnswer((_) async {});
    final store = SecureSecretStore(
      storage,
      storageKey: 'ghostr.viewer.pendingSecret',
    );

    await store.write('nsec1pending');

    verify(
      () => storage.write(
        key: 'ghostr.viewer.pendingSecret',
        value: 'nsec1pending',
      ),
    ).called(1);
  });
}
