package com.adiyutant.app.ui.navigation

import androidx.annotation.StringRes
import com.adiyutant.app.R

/**
 * The five top-level tabs of the app shell.
 *
 * Order matches the approved design (docs/design/today-screen.html tabbar):
 * Сегодня / План / Проекты / Время / Настройки.
 */
enum class Destinations(
    val route: String,
    @param:StringRes val labelRes: Int,
) {
    TODAY("today", R.string.tab_today),
    PLAN("plan", R.string.tab_plan),
    PROJECTS("projects", R.string.tab_projects),
    TIME("time", R.string.tab_time),
    SETTINGS("settings", R.string.tab_settings),
}
