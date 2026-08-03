import 'package:ndk/ndk.dart';

abstract interface class NostrProfileSearchPort {
  /// Profile metadata matching a free-text term, or a direct lookup when the
  /// term is an npub or hex public key.
  Future<List<Metadata>> searchProfiles(String query);
}

/// Null object for environments without a profile search backend.
class NoNostrProfileSearch implements NostrProfileSearchPort {
  const NoNostrProfileSearch();

  @override
  Future<List<Metadata>> searchProfiles(String query) async {
    return const <Metadata>[];
  }
}
