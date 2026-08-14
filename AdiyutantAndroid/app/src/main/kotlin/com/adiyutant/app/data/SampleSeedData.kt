package com.adiyutant.app.data

import com.adiyutant.app.domain.model.BlockAccent
import com.adiyutant.app.domain.model.DayMode
import com.adiyutant.app.domain.model.DayPhase
import com.adiyutant.app.domain.model.PlanMode
import com.adiyutant.app.domain.model.PlanUiState
import com.adiyutant.app.domain.model.PlannerState
import com.adiyutant.app.domain.model.Priority
import com.adiyutant.app.domain.model.ScheduledBlock
import com.adiyutant.app.domain.model.Task
import com.adiyutant.app.domain.yyyyMMdd
import java.time.LocalDate
import java.time.LocalTime
import java.time.YearMonth

/**
 * Seed-данные, эквивалент INIT_UNSCHEDULED + INIT_SCHEDULED из App.jsx
 * (строки 40–76) с датами, относительными к [today] (раздел 13.3 ТЗ).
 */
object SampleSeedData {

    fun build(today: LocalDate): PlannerState {
        val d1 = today.plusDays(1)
        val d2 = today.plusDays(2)
        val m1 = today.minusDays(1)
        val m2 = today.minusDays(2)

        val tasks = listOf(
            task("t1", "Планирование дня", "Adiyutant", 60, Priority.MEDIUM),
            task("t2", "Код ревью PR #42", "Adiyutant", 60, Priority.MEDIUM),
            task("t3", "Deep work: React компоненты", "Adiyutant", 90, Priority.HIGH),
            task("t4", "Sprint Planning", "Adiyutant", 60, Priority.MEDIUM),
            task("t5", "Дизайн-система: токены", "Adiyutant", 80, Priority.MEDIUM),
            task("t6", "Запрос на хостинг", "Инфраструктура", 30, Priority.LOW),
            task("t7", "Ретроспектива спринта", "Adiyutant", 60, Priority.MEDIUM),
            task("t8", "Деплой v2.1", "Adiyutant", 90, Priority.MEDIUM),
            task("t9", "Deep work: архитектура", "Adiyutant", 90, Priority.HIGH),
            task("t10", "Субботний хакатон", "Self Development", 180, Priority.MEDIUM),
            task("t11", "Onboarding встречи", "Adiyutant", 90, Priority.MEDIUM),
            task("t12", "Deep work: бэкенд", "Adiyutant", 120, Priority.HIGH),
            task("t13", "Обед", "", 60, Priority.LOW),
            task("t14", "Code review", "Adiyutant", 45, Priority.MEDIUM),
            task("t15", "Дизайн-сессия", "Adiyutant", 120, Priority.MEDIUM),
            task("t16", "Интеграционные тесты", "Adiyutant", 120, Priority.MEDIUM),
            task("tu1", "Linux namespaces & cgroups", "Обучение", 90, Priority.HIGH, today),
            task("tu2", "English speech practice", "Self Development", 45, Priority.MEDIUM, today),
            task("tu3", "Позвонить в техподдержку", "Личное", 15, Priority.LOW, d1),
            task("tu4", "Обзор статьи по Kubernetes", "Обучение", 30, Priority.MEDIUM, null),
            task("tu5", "Написать тесты для формы", "Adiyutant", 60, Priority.HIGH, d2),
            task("tu6", "Подготовить презентацию", "Adiyutant", 45, Priority.LOW, null, postponed = true),
        )

        val blocks = listOf(
            block("t1", today, "09:00", "10:00", 100, BlockAccent.BLUE),
            block("t2", today, "10:15", "11:15", 100, BlockAccent.BLUE),
            block("t3", today, "11:30", "13:00", 45, BlockAccent.PURPLE),
            block("t4", today, "14:00", "15:00", 0, BlockAccent.BLUE),
            block("t5", today, "15:15", "16:35", 0, BlockAccent.BLUE),
            block("t6", today, "17:00", "17:30", 0, BlockAccent.ORANGE),
            block("t7", d1, "10:00", "11:00", 0, BlockAccent.BLUE),
            block("t8", d1, "13:00", "14:30", 0, BlockAccent.GREEN),
            block("t9", d1, "15:00", "16:30", 0, BlockAccent.PURPLE),
            block("t10", d2, "10:00", "13:00", 0, BlockAccent.PURPLE),
            block("t11", m1, "09:00", "10:30", 100, BlockAccent.BLUE),
            block("t12", m1, "11:00", "13:00", 100, BlockAccent.PURPLE),
            block("t13", m1, "13:00", "14:00", 100, BlockAccent.NONE),
            block("t14", m1, "14:00", "14:45", 100, BlockAccent.BLUE),
            block("t15", m2, "10:00", "12:00", 100, BlockAccent.PURPLE),
            block("t16", m2, "13:00", "15:00", 100, BlockAccent.BLUE),
        )

        return PlannerState(
            todayDate = today,
            tasks = tasks,
            blocks = blocks,
            activeSession = null,
            factSessions = emptyList(),
            dayCheckIn = null,
            dayPhase = DayPhase.DAY_NOT_STARTED,
            dayMode = DayMode.NORMAL,
            dayEnergy = 0,
            todayTop = listOf("t3", "t4", "t5"),
            quickNotesToday = emptyList(),
            endDayReview = null,
            planUi = PlanUiState(
                mode = PlanMode.WEEK,
                selectedDate = today,
                viewMonth = YearMonth.from(today),
            ),
            sheet = null,
            pendingDoElse = null,
        )
    }

    private fun task(
        taskId: String,
        title: String,
        project: String,
        durationMin: Int,
        priority: Priority,
        plannedDate: LocalDate? = null,
        postponed: Boolean = false,
    ): Task = Task(
        taskId = taskId,
        title = title,
        project = project,
        durationMin = durationMin,
        priority = priority,
        plannedDate = plannedDate,
        postponed = postponed,
    )

    private fun block(
        taskId: String,
        date: LocalDate,
        start: String,
        end: String,
        progressPercent: Int,
        accent: BlockAccent,
    ): ScheduledBlock = ScheduledBlock(
        blockId = "s_${taskId}_${yyyyMMdd(date)}",
        taskId = taskId,
        date = date,
        start = LocalTime.parse(start),
        end = LocalTime.parse(end),
        progressPercent = progressPercent,
        accent = accent,
    )
}
