package social.ghostr

import android.content.Context
import android.content.pm.PackageInfo
import android.content.pm.PackageManager
import android.os.Build
import java.io.File
import java.security.MessageDigest

internal class ApkUpdateValidator(
    private val context: Context,
) {
    private val packageManager = context.packageManager

    fun validate(path: String, expectedVersionCode: Long): File {
        val file = privateApk(path)
        val archive = archiveInfo(file)
        requireIdentity(archive)
        requireVersion(archive, expectedVersionCode)
        requireSigner(archive)
        return file
    }

    private fun privateApk(path: String): File {
        val file = File(path).canonicalFile
        val allowed = listOf(context.filesDir, context.cacheDir).any { root ->
            file.isWithin(root.canonicalFile)
        }
        if (!allowed || !file.isFile || file.extension.lowercase() != "apk") {
            throw AppUpdateFailure(
                "invalid_apk_path",
                "The update must be an app-private APK file.",
            )
        }
        return file
    }

    @Suppress("DEPRECATION")
    private fun archiveInfo(file: File): PackageInfo {
        val flags = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            PackageManager.GET_SIGNING_CERTIFICATES
        } else {
            PackageManager.GET_SIGNATURES
        }
        return packageManager.getPackageArchiveInfo(file.path, flags)
            ?: throw AppUpdateFailure("invalid_apk", "Android could not read the update APK.")
    }

    private fun requireIdentity(archive: PackageInfo) {
        if (archive.packageName != context.packageName) {
            throw AppUpdateFailure(
                "package_mismatch",
                "The update APK belongs to a different application.",
            )
        }
    }

    private fun requireVersion(archive: PackageInfo, expected: Long) {
        val installed = packageManager.getPackageInfo(context.packageName, 0)
        if (expected <= installed.versionCodeLong() || archive.versionCodeLong() != expected) {
            throw AppUpdateFailure(
                "version_mismatch",
                "The update APK version is not the expected newer version.",
            )
        }
    }

    private fun requireSigner(archive: PackageInfo) {
        val flags = signingFlags()
        val installed = packageManager.getPackageInfo(context.packageName, flags)
        if (!signerCompatible(archive, installed)) {
            throw AppUpdateFailure(
                "signer_mismatch",
                "The update APK was not signed by the installed application signer.",
            )
        }
    }

    @Suppress("DEPRECATION")
    private fun signerCompatible(archive: PackageInfo, installed: PackageInfo): Boolean {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.P) {
            return signerDigests(archive) == signerDigests(installed)
        }
        val archiveSigning = archive.signingInfo ?: return false
        val installedSigning = installed.signingInfo ?: return false
        if (archiveSigning.hasMultipleSigners() || installedSigning.hasMultipleSigners()) {
            return signerDigests(archive) == signerDigests(installed)
        }
        val installedCurrent = signerDigests(installed)
        val archiveLineage = archiveSigning.signingCertificateHistory
            .map { signature -> sha256(signature.toByteArray()) }
            .toSet()
        return installedCurrent.isNotEmpty() && archiveLineage.containsAll(installedCurrent)
    }

    @Suppress("DEPRECATION")
    private fun signerDigests(info: PackageInfo): Set<String> {
        val signatures = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            info.signingInfo?.apkContentsSigners.orEmpty()
        } else {
            info.signatures.orEmpty()
        }
        return signatures.map { signature -> sha256(signature.toByteArray()) }.toSet()
    }

    @Suppress("DEPRECATION")
    private fun signingFlags(): Int {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            PackageManager.GET_SIGNING_CERTIFICATES
        } else {
            PackageManager.GET_SIGNATURES
        }
    }
}

private fun File.isWithin(root: File): Boolean {
    return path == root.path || path.startsWith(root.path + File.separator)
}

private fun sha256(value: ByteArray): String {
    val digest = MessageDigest.getInstance("SHA-256").digest(value)
    return digest.joinToString("") { byte -> "%02x".format(byte) }
}

@Suppress("DEPRECATION")
private fun PackageInfo.versionCodeLong(): Long {
    return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
        longVersionCode
    } else {
        versionCode.toLong()
    }
}
