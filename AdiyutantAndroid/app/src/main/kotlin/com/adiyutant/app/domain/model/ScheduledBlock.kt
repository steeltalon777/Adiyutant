package com.adiyutant.app.domain.model

import java.time.LocalDate
import java.time.LocalTime

/**
 * Одно размещение задачи во времени (раздел 3.3 ТЗ).
 *
 * [blockId] уникален и непрозрачен (формат не разбирается кодом);
 * same-date reschedule сохраняет [blockId], cross-date move переносит его же.
 * Названия задач НЕ копируются в блок — резолвятся через [taskId].
 */
data class ScheduledBlock(
    val blockId: String,
    val taskId: String,
    val date: LocalDate,
    val start: LocalTime,
    val end: LocalTime,        // end > start
    val progressPercent: Int,  // 0..100; 100 = завершено
    val accent: BlockAccent,   // полоса в timeline
)
