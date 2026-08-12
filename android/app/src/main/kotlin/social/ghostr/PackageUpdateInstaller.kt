package social.ghostr

import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageInstaller
import android.os.Build
import java.io.File

internal data class InstallRequest(
    val path: String,
    val expectedVersionCode: Long,
    val automatic: Boolean,
)

internal class PackageUpdateInstaller(
    private val context: Context,
) {
    private val packageInstaller = context.packageManager.packageInstaller
    private val validator = ApkUpdateValidator(context)

    fun install(request: InstallRequest): Int {
        requireInstallPermission()
        val apk = validator.validate(request.path, request.expectedVersionCode)
        val parameters = sessionParameters(apk, request.automatic)
        val sessionId = packageInstaller.createSession(parameters)
        try {
            writeAndCommit(sessionId, apk)
        } catch (error: Exception) {
            packageInstaller.abandonSession(sessionId)
            throw error
        }
        return sessionId
    }

    fun replace(sessionId: Int, request: InstallRequest): Int {
        packageInstaller.abandonSession(sessionId)
        return install(request)
    }

    private fun requireInstallPermission() {
        val allowed = Build.VERSION.SDK_INT < Build.VERSION_CODES.O ||
            context.packageManager.canRequestPackageInstalls()
        if (!allowed) {
            throw AppUpdateFailure(
                "install_permission_required",
                "Allow Ghostr to install app updates first.",
            )
        }
    }

    private fun sessionParameters(apk: File, automatic: Boolean): PackageInstaller.SessionParams {
        return PackageInstaller.SessionParams(PackageInstaller.SessionParams.MODE_FULL_INSTALL).apply {
            setAppPackageName(context.packageName)
            setSize(apk.length())
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                val action = if (automatic) {
                    PackageInstaller.SessionParams.USER_ACTION_NOT_REQUIRED
                } else {
                    PackageInstaller.SessionParams.USER_ACTION_REQUIRED
                }
                setRequireUserAction(action)
            }
        }
    }

    private fun writeAndCommit(sessionId: Int, apk: File) {
        packageInstaller.openSession(sessionId).use { session ->
            apk.inputStream().use { input ->
                session.openWrite("base.apk", 0, apk.length()).use { output ->
                    input.copyTo(output)
                    session.fsync(output)
                }
            }
            session.commit(statusIntent(sessionId).intentSender)
        }
    }

    private fun statusIntent(sessionId: Int): PendingIntent {
        val intent = Intent(context, InstallStatusReceiver::class.java)
            .putExtra(PackageInstaller.EXTRA_SESSION_ID, sessionId)
        var flags = PendingIntent.FLAG_UPDATE_CURRENT
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            flags = flags or PendingIntent.FLAG_MUTABLE
        }
        return PendingIntent.getBroadcast(context, sessionId, intent, flags)
    }
}
