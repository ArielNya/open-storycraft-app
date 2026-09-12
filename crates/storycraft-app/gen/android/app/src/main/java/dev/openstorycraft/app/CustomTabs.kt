package dev.openstorycraft.app

import android.content.Context
import android.net.Uri
import androidx.browser.customtabs.CustomTabsIntent

/** Device-code OAuth must open auth.x.ai in a Custom Tab, not a WebView. */
object CustomTabs {
    fun open(context: Context, url: String) {
        val builder = CustomTabsIntent.Builder()
        builder.setShowTitle(true)
        builder.build().launchUrl(context, Uri.parse(url))
    }
}
