package social.ghostr

internal data class IncomingVideoShare(
    val path: String,
    val label: String,
    val mimeType: String,
    val sourceKey: IncomingVideoShareCaptureId,
) {
    fun toMap(): Map<String, String> = mapOf(
        "path" to path,
        "label" to label,
        "mimeType" to mimeType,
    )
}
