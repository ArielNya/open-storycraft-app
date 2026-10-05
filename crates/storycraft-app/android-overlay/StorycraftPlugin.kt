package dev.openstorycraft.app

import android.app.Activity
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin

@InvokeArg
class JobArgs {
    var skill: String = "skill"
}

@InvokeArg
class UrlArgs {
    lateinit var url: String
}

/** Rust → Kotlin bridge, registered in src/android.rs. */
@TauriPlugin
class StorycraftPlugin(private val activity: Activity) : Plugin(activity) {
    @Command
    fun startJob(invoke: Invoke) {
        JobForegroundService.start(activity, invoke.parseArgs(JobArgs::class.java).skill)
        invoke.resolve()
    }

    @Command
    fun stopJob(invoke: Invoke) {
        JobForegroundService.stop(activity)
        invoke.resolve()
    }

    @Command
    fun openAuthTab(invoke: Invoke) {
        CustomTabs.open(activity, invoke.parseArgs(UrlArgs::class.java).url)
        invoke.resolve()
    }
}
