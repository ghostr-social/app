package app.ghostr

import android.content.ContentProvider
import android.content.ContentValues
import android.content.pm.ProviderInfo
import android.database.Cursor
import android.database.MatrixCursor
import android.net.Uri
import android.os.ParcelFileDescriptor
import android.provider.OpenableColumns
import java.io.File

internal class ShareTestProvider(
    bytes: ByteArray,
    private val mimeType: String?,
    private val label: String = "shared.mp4",
    private val reportedSize: Long? = bytes.size.toLong(),
    private val hasMetadata: Boolean = true,
) : ContentProvider() {
    private lateinit var content: File
    var openCount = 0
        private set
    private var currentBytes = bytes

    override fun onCreate(): Boolean {
        content = File.createTempFile("provider-video-", ".bin", context!!.cacheDir)
        content.writeBytes(currentBytes)
        return true
    }

    override fun getType(uri: Uri): String? = mimeType

    override fun query(
        uri: Uri,
        projection: Array<out String>?,
        selection: String?,
        selectionArgs: Array<out String>?,
        sortOrder: String?,
    ): Cursor? {
        if (!hasMetadata) return null
        val columns = projection ?: emptyArray()
        return MatrixCursor(columns).apply {
            addRow(columns.map(::columnValue))
        }
    }

    private fun columnValue(column: String): Any? = when (column) {
        OpenableColumns.DISPLAY_NAME -> label
        OpenableColumns.SIZE -> reportedSize
        else -> null
    }

    override fun openFile(uri: Uri, mode: String): ParcelFileDescriptor {
        openCount += 1
        return ParcelFileDescriptor.open(content, ParcelFileDescriptor.MODE_READ_ONLY)
    }

    fun replaceContent(bytes: ByteArray) {
        currentBytes = bytes
        content.writeBytes(bytes)
    }

    override fun insert(uri: Uri, values: ContentValues?): Uri? = null
    override fun delete(uri: Uri, selection: String?, args: Array<out String>?): Int = 0
    override fun update(
        uri: Uri,
        values: ContentValues?,
        selection: String?,
        args: Array<out String>?,
    ): Int = 0
}
