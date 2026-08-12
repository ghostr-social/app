package social.ghostr

import android.content.ContentProvider
import android.content.ContentValues
import android.database.Cursor
import android.database.MatrixCursor
import android.net.Uri
import android.os.ParcelFileDescriptor
import android.provider.OpenableColumns
import java.io.File
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

internal class BlockingShareTestProvider(
    private val bytes: ByteArray,
) : ContentProvider() {
    private lateinit var content: File
    val openStarted = CountDownLatch(1)
    val allowOpen = CountDownLatch(1)
    var openCount = 0
        private set

    override fun onCreate(): Boolean {
        content = File.createTempFile("blocking-provider-", ".mp4", context!!.cacheDir)
        content.writeBytes(bytes)
        return true
    }

    override fun getType(uri: Uri): String = "video/mp4"

    override fun query(
        uri: Uri,
        projection: Array<out String>?,
        selection: String?,
        selectionArgs: Array<out String>?,
        sortOrder: String?,
    ): Cursor {
        val columns = projection ?: emptyArray()
        return MatrixCursor(columns).apply {
            addRow(columns.map(::columnValue))
        }
    }

    override fun openFile(uri: Uri, mode: String): ParcelFileDescriptor {
        openCount += 1
        openStarted.countDown()
        check(allowOpen.await(5, TimeUnit.SECONDS))
        return ParcelFileDescriptor.open(content, ParcelFileDescriptor.MODE_READ_ONLY)
    }

    private fun columnValue(column: String): Any? = when (column) {
        OpenableColumns.DISPLAY_NAME -> "shared.mp4"
        OpenableColumns.SIZE -> bytes.size.toLong()
        else -> null
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
