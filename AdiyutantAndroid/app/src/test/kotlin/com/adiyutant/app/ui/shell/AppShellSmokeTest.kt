package com.adiyutant.app.ui.shell

import com.adiyutant.app.data.SampleSeedData
import com.adiyutant.app.domain.SEED_TODAY
import com.adiyutant.app.domain.deriveHeroState
import com.adiyutant.app.domain.msAt
import com.adiyutant.app.domain.model.DayPhase
import com.adiyutant.app.domain.model.HeroState
import com.adiyutant.app.domain.model.PlanMode
import com.adiyutant.app.ui.navigation.Destinations
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Smoke-тест shell (Phase 2): навигационная структура не меняется,
 * а общий state (seed репозитория) консистентен с контрактами домена.
 * Чистый JVM — без эмулятора.
 */
class AppShellSmokeTest {

    @Test
    fun shellHasExactlyFiveTabsInDesignOrder() {
        val routes = Destinations.entries.map { it.route }
        assertEquals(
            listOf("today", "plan", "projects", "time", "settings"),
            routes,
        )
    }

    @Test
    fun todayIsTheStartDestination() {
        val first = Destinations.entries.first()
        assertEquals(Destinations.TODAY, first)
        assertTrue(first.route == "today")
    }

    @Test
    fun seedStateIsConsistentWithShellContract() {
        val state = SampleSeedData.build(SEED_TODAY)
        assertEquals(22, state.tasks.size)
        assertEquals(16, state.blocks.size)
        assertEquals(listOf("t3", "t4", "t5"), state.todayTop)
        assertEquals(DayPhase.DAY_NOT_STARTED, state.dayPhase)
        assertEquals(PlanMode.WEEK, state.planUi.mode)
        assertEquals(HeroState.DAY_NOT_STARTED, deriveHeroState(state, msAt(SEED_TODAY, 12, 0)))
    }
}
