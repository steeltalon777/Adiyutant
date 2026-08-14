package com.adiyutant.app.domain.model

import java.time.LocalTime

/**
 * Завершённый факт выполнения (раздел 3.4 ТЗ).
 *
 * Plan ≠ Fact: [ScheduledBlock] — намерение, [WorkSession] — реальность.
 * [durationSec] фиксируется один раз в момент завершения.
 */
data class WorkSession(
    val sessionId: String,
    val taskId: String?,         // null для незапланированной работы
    val blockId: String?,
    val parentBlockId: String?,  // исходный плановый блок при «Делаю другое»
    val kind: SessionKind,
    val label: String?,          // для UNPLANNED/BREAK/PERSONAL/OTHER
    val startedAtMs: Long,
    val endedAtMs: Long,
    val durationSec: Long,
    val ease: Ease?,             // только для PLANNED_COMPLETED
)

/**
 * Текущая (активная) сессия (раздел 3.5 ТЗ).
 *
 * [startedAtMs] неизменяем; Pause/Resume меняют только [runningSinceMs],
 * [accumulatedMs], [paused]. Расширение контракта ТЗ (зафиксировано
 * интегратором): [sessionKind] заполняется для kind=UNPLANNED_WORK и
 * переносится в [WorkSession.kind] при завершении.
 */
data class ActiveSession(
    val kind: ActiveKind,
    val taskId: String?,
    val blockId: String?,
    val parentBlockId: String?,
    val label: String?,
    val startedAtMs: Long,
    val runningSinceMs: Long?,   // null, когда paused=true
    val accumulatedMs: Long,
    val paused: Boolean,
    val plannedStart: LocalTime?,
    val plannedEnd: LocalTime?,
    val sessionKind: SessionKind? = null,
)
