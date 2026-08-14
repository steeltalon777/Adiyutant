package com.adiyutant.app.ui.common

import com.adiyutant.app.domain.ScheduleValidator
import com.adiyutant.app.domain.model.ScheduleMode
import java.time.LocalDate
import java.time.LocalTime

/**
 * Чистая JVM-логика ScheduleSheet (контракт раздел 10 ТЗ) — без Compose,
 * тестируется чистым JUnit. Валидация времени/слотов остаётся в
 * [ScheduleValidator] (домен, не дублируется); здесь — только UI-состояние:
 * парсинг полей, выбор дат, шаги времени, label confirm-кнопки.
 */
object ScheduleSheetLogic {

    private val HH_MM = Regex("^(\\d{1,2}):(\\d{2})$")

    /** Парсит «HH:MM» → [LocalTime]; null при любом невалидном вводе (10.3 п.6). */
    fun parseHhMm(input: String): LocalTime? {
        val match = HH_MM.matchEntire(input.trim()) ?: return null
        val hour = match.groupValues[1].toIntOrNull() ?: return null
        val minute = match.groupValues[2].toIntOrNull() ?: return null
        if (hour > 23 || minute > 59) return null
        return LocalTime.of(hour, minute)
    }

    /** Тип даты-чипа для лейбла «Сегодня»/«Завтра»/«Пт · 14 авг». */
    enum class DateChoiceKind { TODAY, TOMORROW, OTHER }

    fun dateChoiceKind(date: LocalDate, today: LocalDate): DateChoiceKind = when (date) {
        today -> DateChoiceKind.TODAY
        today.plusDays(1) -> DateChoiceKind.TOMORROW
        else -> DateChoiceKind.OTHER
    }

    /**
     * Чипы дат [sourceDate, today, tomorrow, today+2] с дедупом, в этом
     * порядке (10.4). Не содержит null — все даты всегда определены.
     */
    fun dateChoices(sourceDate: LocalDate, today: LocalDate): List<LocalDate> {
        val out = mutableListOf<LocalDate>()
        listOf(sourceDate, today, today.plusDays(1), today.plusDays(2)).forEach { d ->
            if (!out.contains(d)) out.add(d)
        }
        return out
    }

    /** Label confirm-кнопки (10.3 п.8): Запланировать / Сохранить / Перенести. */
    enum class ConfirmLabel { SCHEDULE, SAVE, MOVE }

    fun confirmLabel(mode: ScheduleMode, date: LocalDate, sourceDate: LocalDate): ConfirmLabel = when {
        mode == ScheduleMode.SCHEDULE -> ConfirmLabel.SCHEDULE
        date == sourceDate -> ConfirmLabel.SAVE
        else -> ConfirmLabel.MOVE
    }

    /** Первое нажатие ± при пустом времени: 09:00 + duration (10.4). */
    fun initialFromEmpty(durationMin: Int): Pair<LocalTime, LocalTime> {
        val start = ScheduleValidator.WORKDAY_START
        return start to start.plusMinutes(durationMin.toLong())
    }

    /** Сдвиг start на [deltaMin]; end = новый start + duration. */
    fun shifted(start: LocalTime, durationMin: Int, deltaMin: Int): Pair<LocalTime, LocalTime> {
        val newStart = start.plusMinutes(deltaMin.toLong())
        return newStart to newStart.plusMinutes(durationMin.toLong())
    }

    /** End редактируется только если > start (10.4). */
    fun canSetEnd(start: LocalTime?, end: LocalTime?): Boolean =
        start != null && end != null && end.isAfter(start)

    /** end = start + duration (после ручного ввода start). */
    fun endForStart(start: LocalTime, durationMin: Int): LocalTime =
        start.plusMinutes(durationMin.toLong())
}
