package social.ghostr

import android.content.Context
import android.net.Uri
import java.io.File
import java.io.IOException
import java.io.InputStream
import java.io.OutputStream

internal class IncomingVideoShareCache(context: Context) {
    private val contentResolver = context.contentResolver
    private val cacheDirectory = context.cacheDir

    fun requireAllowedSize(sizeBytes: Long?) {
        if (sizeBytes != null && sizeBytes !in 1L..MAX_COPY_BYTES) {
            throw IOException()
        }
    }

    fun copy(uri: Uri, extension: String): File {
        val target = File.createTempFile(
            CACHE_PREFIX,
            ".$extension",
            cacheDirectory,
        )
        try {
            copyContent(uri, target)
            return target
        } catch (error: Exception) {
            target.delete()
            throw error
        }
    }

    fun deleteStaleFiles(retainedPaths: Set<String>) {
        val cutoff = System.currentTimeMillis() - STALE_FILE_AGE_MILLIS
        cacheDirectory.listFiles { file ->
            file.path !in retainedPaths &&
                file.name.startsWith(CACHE_PREFIX) &&
                file.lastModified() <= cutoff
        }?.forEach(File::delete)
    }

    fun deleteOwnedFile(path: String): Boolean {
        return try {
            val file = File(path).canonicalFile
            isOwnedFile(file) && file.delete()
        } catch (_: IOException) {
            false
        }
    }

    fun isOwnedFile(path: String): Boolean {
        return try {
            isOwnedFile(File(path).canonicalFile)
        } catch (_: IOException) {
            false
        }
    }

    private fun isOwnedFile(file: File): Boolean {
        return file.parentFile == cacheDirectory.canonicalFile &&
            file.name.startsWith(CACHE_PREFIX)
    }

    private fun copyContent(uri: Uri, target: File) {
        val input = contentResolver.openInputStream(uri) ?: throw IOException()
        input.use { source ->
            target.outputStream().buffered().use { output ->
                copyBounded(source, output)
            }
        }
    }

    private fun copyBounded(input: InputStream, output: OutputStream) {
        val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
        var total = 0L
        while (true) {
            val count = input.read(buffer)
            if (count < 0) break
            total += count
            if (total > MAX_COPY_BYTES) throw IOException()
            output.write(buffer, 0, count)
        }
        if (total == 0L) throw IOException()
    }

    private companion object {
        const val CACHE_PREFIX = "incoming-video-"
        const val MAX_COPY_BYTES = 1_073_741_824L
        const val STALE_FILE_AGE_MILLIS = 24L * 60L * 60L * 1000L
    }
}
