import 'package:mocktail/mocktail.dart';
import 'package:ndk/entities.dart';

import 'ndk_mocks.dart';

/// Stubs the NDK surfaces the outbox directory reads: the signed-in
/// account, its contact list, and per-author NIP-65 relay lists.
class OutboxDirectoryHarness {
  OutboxDirectoryHarness({required this.viewer}) {
    when(() => ndk.accounts).thenReturn(accounts);
    when(() => ndk.follows).thenReturn(follows);
    when(() => ndk.userRelayLists).thenReturn(userRelayLists);
    when(accounts.getPublicKey).thenReturn(viewer);
    when(() => userRelayLists.loadMissingRelayListsFromNip65OrNip02(any()))
        .thenAnswer((_) async {});
    when(() => userRelayLists.getSingleUserRelayList(any()))
        .thenAnswer((_) async => null);
  }

  final String viewer;
  final MockNdk ndk = MockNdk();
  final MockAccounts accounts = MockAccounts();
  final MockFollows follows = MockFollows();
  final MockUserRelayLists userRelayLists = MockUserRelayLists();

  void stubContacts(List<String> contacts) {
    when(() => follows.getContactList(viewer)).thenAnswer(
      (_) async => ContactList(pubKey: viewer, contacts: contacts),
    );
  }

  void stubRelayList(String pubKey, Map<String, ReadWriteMarker> relays) {
    when(() => userRelayLists.getSingleUserRelayList(pubKey)).thenAnswer(
      (_) async => UserRelayList(
        pubKey: pubKey,
        relays: relays,
        createdAt: 0,
        refreshedTimestamp: 0,
      ),
    );
  }
}
