package social.ghostr

import android.app.Activity
import io.flutter.plugin.common.BinaryMessenger
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import java.util.concurrent.Executors

internal class AndroidAppUpdateBridge(
    private val activity: Activity,
    messenger: BinaryMessenger,
) : MethodChannel.MethodCallHandler {
    private val channel = MethodChannel(messenger, channelName)
    private val inspector = AndroidAppInspector(activity)
    private val worker = Executors.newSingleThreadExecutor()
    private val installer = PackageUpdateInstaller(activity.applicationContext)
    private val statusStore = InstallStatusStore(activity.applicationContext)
    @Volatile
    private var disposed = false

    init {
        channel.setMethodCallHandler(this)
        InstallStatusDispatcher.attach(activity, ::sendStatus)
    }

    override fun onMethodCall(call: MethodCall, result: MethodChannel.Result) {
        try {
            when (call.method) {
                "getInstalledApp" -> result.success(inspector.installedApp())
                "getNetworkAccess" -> result.success(inspector.networkAccess())
                "canRequestInstalls" -> result.success(inspector.canRequestInstalls())
                "openInstallPermissionSettings" -> openPermissionSettings(result)
                "readInstallStatus" -> readInstallStatus(call.arguments, result)
                "install" -> beginInstall(call.arguments, result)
                "replaceInstall" -> replaceInstall(call.arguments, result)
                else -> result.notImplemented()
            }
        } catch (failure: AppUpdateFailure) {
            result.error(failure.code, failure.message, null)
        } catch (error: Exception) {
            result.error("app_update_error", error.message, null)
        }
    }

    fun dispose() {
        disposed = true
        channel.setMethodCallHandler(null)
        InstallStatusDispatcher.detach(activity)
        worker.shutdown()
    }

    private fun openPermissionSettings(result: MethodChannel.Result) {
        inspector.openInstallPermissionSettings()
        result.success(null)
    }

    private fun beginInstall(arguments: Any?, result: MethodChannel.Result) {
        val request = installRequest(arguments)
        runInstall(result) { installer.install(request) }
    }

    private fun replaceInstall(arguments: Any?, result: MethodChannel.Result) {
        val values = arguments as? Map<*, *>
            ?: throw AppUpdateFailure("invalid_arguments", "Install arguments are required.")
        val sessionId = values.requiredInt("sessionId")
        val request = installRequest(values)
        runInstall(result) { installer.replace(sessionId, request) }
    }

    private fun runInstall(result: MethodChannel.Result, action: () -> Int) {
        worker.execute {
            try {
                val sessionId = action()
                reply { result.success(sessionId) }
            } catch (failure: AppUpdateFailure) {
                reply { result.error(failure.code, failure.message, null) }
            } catch (error: Exception) {
                reply { result.error("install_failed", error.message, null) }
            }
        }
    }

    private fun readInstallStatus(arguments: Any?, result: MethodChannel.Result) {
        val values = arguments as? Map<*, *>
            ?: throw AppUpdateFailure("invalid_arguments", "Status arguments are required.")
        val sessionId = values.requiredInt("sessionId")
        result.success(statusStore.read(sessionId)?.payload())
    }

    private fun installRequest(arguments: Any?): InstallRequest {
        val values = arguments as? Map<*, *>
            ?: throw AppUpdateFailure("invalid_arguments", "Install arguments are required.")
        return InstallRequest(
            path = values.requiredString("path"),
            expectedVersionCode = values.requiredLong("expectedVersionCode"),
            automatic = values.requiredBoolean("automatic"),
        )
    }

    private fun sendStatus(status: InstallStatus) {
        if (!disposed) channel.invokeMethod("installStatus", status.payload())
    }

    private fun reply(action: () -> Unit) {
        activity.runOnUiThread {
            if (!disposed) action()
        }
    }

    private companion object {
        const val channelName = "social.ghostr/app_update/v1"
    }
}

private fun Map<*, *>.requiredString(name: String): String {
    return this[name] as? String
        ?: throw AppUpdateFailure("invalid_arguments", "$name must be text.")
}

private fun Map<*, *>.requiredLong(name: String): Long {
    return (this[name] as? Number)?.toLong()
        ?: throw AppUpdateFailure("invalid_arguments", "$name must be an integer.")
}

private fun Map<*, *>.requiredInt(name: String): Int {
    return (this[name] as? Number)?.toInt()
        ?: throw AppUpdateFailure("invalid_arguments", "$name must be an integer.")
}

private fun Map<*, *>.requiredBoolean(name: String): Boolean {
    return this[name] as? Boolean
        ?: throw AppUpdateFailure("invalid_arguments", "$name must be true or false.")
}
