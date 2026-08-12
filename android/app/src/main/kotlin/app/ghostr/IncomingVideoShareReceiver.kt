package app.ghostr

import android.content.ContentResolver
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.OpenableColumns
import java.util.Locale

internal class IncomingVideoShareReceiver(
    context: Context,
    leaseId: IncomingVideoShareLeaseId = IncomingVideoShareLeaseId.create(),
) {
    private val contentResolver = context.contentResolver
    private val cache = IncomingVideoShareCache(context)
    private val store = IncomingVideoShareStore(context, cache, leaseId)

    fun deleteStaleCacheFiles() {
        cache.deleteStaleFiles(store.retainedPaths())
    }

    fun restorePending(): IncomingVideoShare? = store.load()

    fun isHandled(captured: CapturedIncomingVideoIntent): Boolean {
        return captured.captureId?.let(store::isHandled) == true
    }

    fun acknowledge(path: String) {
        store.acknowledge(path)
    }

    fun release(path: String) {
        store.release(path)
        cache.deleteOwnedFile(path)
    }

    fun receive(captured: CapturedIncomingVideoIntent): IncomingVideoShare? {
        try {
            return receiveChecked(captured)
        } catch (_: IncomingVideoShareException) {
            throw IncomingVideoShareException()
        } catch (_: Exception) {
            throw IncomingVideoShareException()
        }
    }

    private fun receiveChecked(captured: CapturedIncomingVideoIntent): IncomingVideoShare? {
        val intent = captured.intent
        if (intent.action != Intent.ACTION_SEND) return null
        val captureId = captured.captureId ?: throw IncomingVideoShareException()
        val uri = streamUri(intent) ?: throw IncomingVideoShareException()
        if (uri.scheme != ContentResolver.SCHEME_CONTENT) {
            throw IncomingVideoShareException()
        }
        val metadata = metadata(uri)
        val sourceLabel = metadata.label ?: uri.lastPathSegment
        val mimeType = resolvedMimeType(intent.type, uri, sourceLabel)
            ?: throw IncomingVideoShareException()
        cache.requireAllowedSize(metadata.sizeBytes)
        val label = sanitizedLabel(sourceLabel, mimeType)
        val file = cache.copy(uri, extensionFor(mimeType))
        val share = IncomingVideoShare(file.path, label, mimeType, captureId)
        try {
            val previous = store.save(share)
            if (previous?.path != share.path) previous?.let { release(it.path) }
            return share
        } catch (error: Exception) {
            cache.deleteOwnedFile(share.path)
            throw error
        }
    }

    private fun resolvedMimeType(
        declared: String?,
        uri: Uri,
        label: String?,
    ): String? {
        return supportedMimeType(declared)
            ?: supportedMimeType(contentResolver.getType(uri))
            ?: mimeTypeFromLabel(label)
    }

    private fun supportedMimeType(raw: String?): String? {
        val value = raw?.substringBefore(';')?.trim()?.lowercase(Locale.ROOT)
        return value?.takeIf(SUPPORTED_MIME_TYPES::contains)
    }

    private fun mimeTypeFromLabel(label: String?): String? {
        val extension = label?.substringAfterLast('.', missingDelimiterValue = "")
            ?.lowercase(Locale.ROOT)
        return extension?.let(MIME_BY_EXTENSION::get)
    }

    private fun streamUri(intent: Intent): Uri? {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            intent.getParcelableExtra(Intent.EXTRA_STREAM, Uri::class.java)
        } else {
            legacyStreamUri(intent)
        }
    }

    @Suppress("DEPRECATION")
    private fun legacyStreamUri(intent: Intent): Uri? {
        return intent.getParcelableExtra(Intent.EXTRA_STREAM)
    }

    private fun metadata(uri: Uri): SharedContentMetadata {
        return contentResolver.query(
            uri,
            arrayOf(OpenableColumns.DISPLAY_NAME, OpenableColumns.SIZE),
            null,
            null,
            null,
        )?.use { cursor ->
            if (!cursor.moveToFirst()) return@use SharedContentMetadata()
            val labelIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
            val sizeIndex = cursor.getColumnIndex(OpenableColumns.SIZE)
            val label = if (labelIndex >= 0) cursor.getString(labelIndex) else null
            val size = if (sizeIndex >= 0 && !cursor.isNull(sizeIndex)) {
                cursor.getLong(sizeIndex).takeIf { it >= 0L }
            } else null
            SharedContentMetadata(label, size)
        } ?: SharedContentMetadata()
    }

    private fun sanitizedLabel(raw: String?, mimeType: String): String {
        val candidate = raw.orEmpty()
            .substringAfterLast('/')
            .substringAfterLast('\\')
            .filter { !it.isISOControl() }
            .trim()
            .take(MAX_LABEL_LENGTH)
        return candidate.takeUnless { it.isBlank() || it == "." || it == ".." }
            ?: fallbackLabel(mimeType)
    }

    private fun fallbackLabel(mimeType: String): String {
        return "shared-video.${extensionFor(mimeType)}"
    }

    private fun extensionFor(mimeType: String): String {
        return MIME_BY_EXTENSION.entries.firstOrNull { it.value == mimeType }?.key
            ?: DEFAULT_EXTENSION
    }

    private data class SharedContentMetadata(
        val label: String? = null,
        val sizeBytes: Long? = null,
    )

    private companion object {
        val MIME_BY_EXTENSION = mapOf(
            "mp4" to "video/mp4",
            "m4v" to "video/x-m4v",
            "mov" to "video/quicktime",
            "webm" to "video/webm",
            "mkv" to "video/x-matroska",
            "3gp" to "video/3gpp",
        )
        val SUPPORTED_MIME_TYPES = MIME_BY_EXTENSION.values.toSet()
        const val DEFAULT_EXTENSION = "mp4"
        const val MAX_LABEL_LENGTH = 160
    }
}

internal class IncomingVideoShareException : Exception()
