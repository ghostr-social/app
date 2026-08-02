import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/storage/secure_secret_store.dart';
import 'package:mocktail/mocktail.dart';

import '../support/mock_flutter_secure_storage.dart';

void main() {
  test('stores, reads, and clears the nsec under one private key', () async {
    final storage = MockFlutterSecureStorage();
    when(
      () => storage.write(key: any(named: 'key'), value: any(named: 'value')),
    ).thenAnswer((_) async {});
    when(
      () => storage.read(key: any(named: 'key')),
    ).thenAnswer((_) async => 'nsec1secret');
    when(
      () => storage.delete(key: any(named: 'key')),
    ).thenAnswer((_) async {});
    final store = SecureSecretStore(storage);

    await store.write('nsec1secret');
    expect(await store.read(), 'nsec1secret');
    await store.clear();

    verify(
      () => storage.write(
        key: 'ghostr.viewer.secret',
        value: 'nsec1secret',
      ),
    ).called(1);
    verify(() => storage.read(key: 'ghostr.viewer.secret')).called(1);
    verify(() => storage.delete(key: 'ghostr.viewer.secret')).called(1);
  });
}
