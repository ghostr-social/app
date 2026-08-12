package social.ghostr

import android.content.Context
import android.content.SharedPreferences
import java.io.File
import java.io.IOException

internal class IncomingVideoShareStore(
    context: Context,
    private val cache: IncomingVideoShareCache,
    private val leaseId: IncomingVideoShareLeaseId = IncomingVideoShareLeaseId.create(),
) {
    private val preferences: SharedPreferences = context.getSharedPreferences(
        STORE_NAME,
        Context.MODE_PRIVATE,
    )

    fun load(): IncomingVideoShare? {
        val share = savedShare() ?: return null
        if (cache.isOwnedFile(share.path) && File(share.path).isFile) return share
        release(share.path)
        return null
    }

    fun retainedPaths(): Set<String> {
        val paths = ownedPaths().toMutableSet()
        savedShare()?.path?.let(paths::add)
        return paths.filterTo(mutableSetOf()) {
            cache.isOwnedFile(it) && File(it).isFile
        }
    }

    fun save(share: IncomingVideoShare): IncomingVideoShare? {
        val previous = load()
        val saved = preferences.edit()
            .putString(PATH, share.path)
            .putString(LABEL, share.label)
            .putString(MIME_TYPE, share.mimeType)
            .putString(SOURCE_KEY, share.sourceKey.value)
            .putString(LEASE_ID, leaseId.value)
            .putStringSet(OWNED_PATHS, ownedPaths() + share.path)
            .commit()
        if (!saved) throw IOException()
        return previous
    }

    fun acknowledge(path: String) {
        val sourceKey = pendingSourceKey(path) ?: return
        val saved = removePending(preferences.edit())
            .putString(HANDLED_SOURCE_KEY, sourceKey)
            .putString(LEASE_ID, leaseId.value)
            .putStringSet(OWNED_PATHS, ownedPaths() + path)
            .commit()
        if (!saved) throw IOException()
    }

    fun release(path: String) {
        val sourceKey = pendingSourceKey(path)
        var editor = preferences.edit()
            .putString(LEASE_ID, leaseId.value)
            .putStringSet(OWNED_PATHS, ownedPaths() - path)
        if (sourceKey != null) {
            editor = removePending(editor).putString(HANDLED_SOURCE_KEY, sourceKey)
        }
        if (!editor.commit()) throw IOException()
    }

    fun isHandled(captureId: IncomingVideoShareCaptureId): Boolean {
        return preferences.getString(HANDLED_SOURCE_KEY, null) == captureId.value
    }

    fun clear() {
        if (!preferences.edit().clear().commit()) throw IOException()
    }

    private fun ownedPaths(): Set<String> {
        val paths = preferences.getStringSet(OWNED_PATHS, emptySet()).orEmpty().toSet()
        val storedLease = preferences.getString(LEASE_ID, null)
        return paths.takeIf { storedLease == leaseId.value }.orEmpty()
    }

    private fun pendingSourceKey(path: String): String? {
        if (preferences.getString(PATH, null) != path) return null
        return preferences.getString(SOURCE_KEY, null)
    }

    private fun removePending(editor: SharedPreferences.Editor): SharedPreferences.Editor {
        return editor.remove(PATH).remove(LABEL).remove(MIME_TYPE).remove(SOURCE_KEY)
    }

    private fun savedShare(): IncomingVideoShare? {
        val path = preferences.getString(PATH, null) ?: return null
        val label = preferences.getString(LABEL, null) ?: return null
        val mimeType = preferences.getString(MIME_TYPE, null) ?: return null
        val sourceKey = preferences.getString(SOURCE_KEY, null)
            ?.let(::IncomingVideoShareCaptureId) ?: return null
        return IncomingVideoShare(path, label, mimeType, sourceKey)
    }

    private companion object {
        const val STORE_NAME = "incoming_video_share"
        const val PATH = "path"
        const val LABEL = "label"
        const val MIME_TYPE = "mime_type"
        const val SOURCE_KEY = "source_key"
        const val HANDLED_SOURCE_KEY = "handled_source_key"
        const val LEASE_ID = "lease_id"
        const val OWNED_PATHS = "owned_paths"
    }
}
