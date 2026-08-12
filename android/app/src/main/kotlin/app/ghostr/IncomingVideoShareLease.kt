package app.ghostr

import java.util.UUID

@JvmInline
internal value class IncomingVideoShareLeaseId(val value: String) {
    companion object {
        fun create(): IncomingVideoShareLeaseId {
            return IncomingVideoShareLeaseId(UUID.randomUUID().toString())
        }
    }
}
