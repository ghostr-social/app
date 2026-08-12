enum NetworkConnection { offline, wifi, other }

abstract interface class NetworkConnectionPort {
  Future<NetworkConnection> readConnection();
}
