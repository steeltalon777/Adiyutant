package com.adiyutant.app.ui.shell

import com.adiyutant.app.ui.navigation.Destinations
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

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
}
