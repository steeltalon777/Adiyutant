package com.adiyutant.app.core.model

import java.time.LocalDate

/**
 * UI-facing snapshot of the current day.
 *
 * Intentionally a thin mirror of what the Core facade exposes
 * (`TodayViewDto`, `CurrentActivityDto`). Kept minimal for the shell;
 * expanded only when the design system finalizes.
 */
data class TodaySnapshot(
    val date: LocalDate,
    val mode: String,
    val energy: String,
    val mood: String,
    val checkInCount: Int,
    val hasMorningCheckIn: Boolean,
    val hasEveningCheckIn: Boolean,
    val habitCount: Int,
    val habitEventsDone: Int,
    val hasPlan: Boolean,
    val planTitle: String,
    val planItemCount: Int,
)
