BEGIN {
  threshold["event_identity.rs"] = 100
  threshold["native_models.rs"] = 100
  threshold["event_index.rs"] = 100
  threshold["event_indexer.rs"] = 100
  threshold["hls_manifest.rs"] = 100
  threshold["hls_manifest_attributes.rs"] = 100
  threshold["hls_manifest_tags.rs"] = 100
  threshold["hls_resource_capability.rs"] = 100
  threshold["hls_session_state.rs"] = 95
  threshold["hls_session_types.rs"] = 100
  threshold["hls_sessions.rs"] = 95
  threshold["hls_playback_gateway.rs"] = 95
  threshold["hls_http_gateway.rs"] = 95
  threshold["native_blob_integrity.rs"] = 95
  threshold["native_blob_store.rs"] = 95
  threshold["native_candidate_round.rs"] = 100
  threshold["native_cache_capacity.rs"] = 100
  threshold["native_cache_digest.rs"] = 100
  threshold["native_cache_directory.rs"] = 100
  threshold["native_cache_failure.rs"] = 100
  threshold["native_cache_fetch.rs"] = 100
  threshold["native_cache_priority.rs"] = 100
  threshold["native_cache_transfer.rs"] = 95
  threshold["native_deletions.rs"] = 100
  threshold["native_download_candidates.rs"] = 100
  threshold["native_download_group.rs"] = 100
  threshold["native_download_state.rs"] = 100
  threshold["native_download_updates.rs"] = 100
  threshold["native_media_metadata.rs"] = 100
  threshold["native_partial_store.rs"] = 95
  threshold["native_text.rs"] = 100
  threshold["outbound_media_client.rs"] = 95
  threshold["public_dns_resolver.rs"] = 95
  threshold["public_media_address.rs"] = 100
  threshold["ffi_models.rs"] = 95
  threshold["gateway_runtime.rs"] = 95
  threshold["http_gateway.rs"] = 95
  threshold["native_cache.rs"] = 95
  threshold["native_gateway.rs"] = 95
  threshold["video_manager.rs"] = 95
}

/^SF:/ {
  path = substr($0, 4)
  parts = split(path, segments, "/")
  file = segments[parts]
  hit = 0
  total = 0
  next
}

/^DA:/ && file in threshold {
  split(substr($0, 4), data, ",")
  total++
  if (data[2] > 0) {
    hit++
  }
  next
}

/^end_of_record/ && file in threshold {
  seen[file] = 1
  coverage = total == 0 ? 0 : hit * 100 / total
  printf("Native line coverage %s: %.2f%% (%d/%d; required %.0f%%)\n",
         file, coverage, hit, total, threshold[file])
  if (coverage + 0.000001 < threshold[file]) {
    failed = 1
  }
}

END {
  for (required in threshold) {
    if (!(required in seen)) {
      printf("Missing native coverage record for %s\n", required)
      failed = 1
    }
  }
  exit failed
}
