package com.adiyutant.app.domain.model

import java.time.LocalDate

/**
 * Логическая задача (раздел 3.2 ТЗ).
 *
 * [taskId] — стабильная идентичность, никогда не меняется. Единый каталог:
 * title/project/durationMin/priority переносятся между представлениями без
 * изменений. Single-entity rule: если существует [ScheduledBlock] с этим
 * [taskId], то [plannedDate] обязан быть `null` (дата выводится из блока).
 */
data class Task(
    val taskId: String,
    val title: String,
    val project: String,         // "" = без проекта
    val durationMin: Int,        // плановая длительность, > 0
    val priority: Priority,
    val plannedDate: LocalDate?, // null → «Без даты»; задана без блока → «Есть дата, нет времени»
    val postponed: Boolean,      // true → группа «Отложенные»
)
