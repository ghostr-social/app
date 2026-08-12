package social.ghostr

import android.content.Context

internal class InstallStatusStore(context: Context) {
    private val preferences = context.getSharedPreferences(
        "app-update-install-status",
        Context.MODE_PRIVATE,
    )

    fun read(sessionId: Int): InstallStatus? {
        val prefix = "$sessionId."
        val status = preferences.getString(prefix + STATUS, null) ?: return null
        val message = preferences.getString(prefix + MESSAGE, null)
        val platformCode = if (preferences.contains(prefix + CODE)) {
            preferences.getInt(prefix + CODE, 0)
        } else {
            null
        }
        return InstallStatus(sessionId, status, message, platformCode)
    }

    fun write(status: InstallStatus) {
        val prefix = "${status.sessionId}."
        preferences.edit().apply {
            putString(prefix + STATUS, status.status)
            putNullableString(prefix + MESSAGE, status.message)
            putNullableInt(prefix + CODE, status.platformCode)
            commit()
        }
    }

    private companion object {
        const val STATUS = "status"
        const val MESSAGE = "message"
        const val CODE = "code"
    }
}

private fun android.content.SharedPreferences.Editor.putNullableString(
    key: String,
    value: String?,
) {
    if (value == null) remove(key) else putString(key, value)
}

private fun android.content.SharedPreferences.Editor.putNullableInt(
    key: String,
    value: Int?,
) {
    if (value == null) remove(key) else putInt(key, value)
}
