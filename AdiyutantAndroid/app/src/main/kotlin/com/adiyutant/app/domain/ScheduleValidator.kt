package com.adiyutant.app.domain

import com.adiyutant.app.domain.model.ScheduledBlock
import java.time.LocalTime

/** Ошибка валидации планирования (раздел 10.3 ТЗ). */
enum class ValidationError {
    OUT_OF_RANGE,
    OVERLAP,
}

/**
 * Чистые функции валидации расписания (раздел 10.3 ТЗ).
 * Рабочий день 09:00–19:00, пересечение по правилу
 * `other.start < candidateEnd && candidateStart < other.end`.
 */
object ScheduleValidator {

    val WORKDAY_START: LocalTime = LocalTime.of(9, 0)
    val WORKDAY_END: LocalTime = LocalTime.of(19, 0)

    /**
     * @return null — валидно, иначе [ValidationError].
     * [excludeBlockId] исключает собственный блок из расчёта пересечений (RESCHEDULE).
     */
    fun validate(
        start: LocalTime,
        end: LocalTime,
        existing: List<ScheduledBlock>,
        excludeBlockId: String?,
    ): ValidationError? {
        if (start.isBefore(WORKDAY_START) || end.isAfter(WORKDAY_END) || !end.isAfter(start)) {
            return ValidationError.OUT_OF_RANGE
        }
        val conflict = existing.any { other ->
            other.blockId != excludeBlockId && other.start < end && start < other.end
        }
        return if (conflict) ValidationError.OVERLAP else null
    }

    /** Свободные слоты: шаг 30 мин от 09:00 до 19:00-duration, без конфликтов. */
    fun freeSlots(
        existing: List<ScheduledBlock>,
        durationMin: Int,
        excludeBlockId: String?,
    ): List<Pair<LocalTime, LocalTime>> {
        val slots = mutableListOf<Pair<LocalTime, LocalTime>>()
        var m = 9 * 60
        while (m < 19 * 60) {
            val e = m + durationMin
            if (e > 19 * 60) break
            val start = minToTime(m)
            val end = minToTime(e)
            val conflict = existing.any { other ->
                other.blockId != excludeBlockId && other.start < end && start < other.end
            }
            if (!conflict) slots.add(start to end)
            m += 30
        }
        return slots
    }

    /** Принадлежность интервала рабочему дню и end > start. */
    fun isWorkdayTime(start: LocalTime, end: LocalTime): Boolean =
        !start.isBefore(WORKDAY_START) && !end.isAfter(WORKDAY_END) && end.isAfter(start)
}
