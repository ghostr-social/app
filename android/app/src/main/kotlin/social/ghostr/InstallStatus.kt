package social.ghostr

internal data class InstallStatus(
    val sessionId: Int,
    val status: String,
    val message: String? = null,
    val platformCode: Int? = null,
) {
    fun payload(): Map<String, Any> = buildMap {
        put("sessionId", sessionId)
        put("status", status)
        message?.let { put("message", it) }
        platformCode?.let { put("platformCode", it) }
    }
}
