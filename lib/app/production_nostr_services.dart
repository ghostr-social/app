import 'package:ghostr/app/broadcast_transport_selection.dart';
import 'package:ghostr/app/feed_pipeline_flag.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/features/publish/data/nostr_video_publisher.dart';
import 'package:ghostr/features/publish/domain/nostr_video_publisher_port.dart';
import 'package:ghostr/features/session/domain/nostr_session_port.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/social/domain/nostr_social_port.dart';
import 'package:ghostr/features/social/domain/signed_event_broadcast_port.dart';
import 'package:ghostr/features/video_catalog/data/nostr_profile_search_port.dart';
import 'package:ghostr/platform/nostr/build_ndk.dart';
import 'package:ghostr/platform/nostr/ndk_blossom_video_uploader.dart';
import 'package:ghostr/platform/nostr/ndk_broadcast_adapter.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_client.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_profile_search.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_session.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_social.dart';
import 'package:ghostr/platform/nostr/rust_broadcast_adapter.dart';
import 'package:ndk/ndk.dart';

typedef ProductionNdkBuilder = Ndk Function(List<RelayUrl> relays);

ProductionNostrServices buildProductionNostrServices(
  AppSettings settings, {
  ProductionNdkBuilder ndkBuilder = buildNdk,
  FeedPipelineFlag feedFlag = const FeedPipelineFlag(),
}) {
  final ndk = ndkBuilder(settings.relays);
  final eventClient = NdkNostrEventClient(ndk: ndk, relays: settings.relays);
  final broadcast = selectBroadcastTransport(
    mode: feedFlag.mode,
    ndk: NdkBroadcastAdapter(ndk: ndk, relays: settings.relays),
    rust: RustBroadcastAdapter.new,
  );
  return ProductionNostrServices(
    ndk,
    ProductionNostrAdapters(
      NdkNostrSession(ndk),
      NdkNostrSocial(
        ndk: ndk,
        relays: settings.relays,
        broadcast: broadcast,
      ),
      broadcast: broadcast,
    ),
    eventClient,
    NostrVideoPublisher(
      eventClient: eventClient,
      mediaUploader: NdkBlossomVideoUploader(
        ndk: ndk,
        servers: settings.blossomServers,
      ),
    ),
    profileSearch: NdkNostrProfileSearch(
      ndk,
      searchRelays: settings.searchRelays,
    ),
  );
}

class ProductionNostrServices {
  const ProductionNostrServices(
    this.ndk,
    this.adapters,
    this.eventClient,
    this.publisher, {
    NostrProfileSearchPort? profileSearch,
  }) : _profileSearch = profileSearch;

  final Ndk ndk;
  final ProductionNostrAdapters adapters;
  final NostrEventClient eventClient;
  final NostrVideoPublisherPort publisher;
  final NostrProfileSearchPort? _profileSearch;

  NostrProfileSearchPort get profileSearch =>
      _profileSearch ?? const NoNostrProfileSearch();
}

class ProductionNostrAdapters {
  const ProductionNostrAdapters(this.session, this.social, {this.broadcast});

  final NostrSessionPort session;
  final NostrSocialPort social;

  /// The transport [social] publishes through, recorded so the cutover
  /// can be asserted and later shared with the event client. Null when
  /// adapters are composed without a write path.
  final SignedEventBroadcastPort? broadcast;
}
