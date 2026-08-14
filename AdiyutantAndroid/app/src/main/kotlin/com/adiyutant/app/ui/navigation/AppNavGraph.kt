package com.adiyutant.app.ui.navigation

import androidx.compose.runtime.Composable
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.adiyutant.app.ui.feature.plan.PlanScreen
import com.adiyutant.app.ui.feature.projects.ProjectsScreen
import com.adiyutant.app.ui.feature.settings.SettingsScreen
import com.adiyutant.app.ui.feature.time.TimeScreen
import com.adiyutant.app.ui.feature.today.TodayScreen

/**
 * Navigation graph for the five-tab shell (ADR-0002, Navigation Compose).
 * Each tab maps 1:1 to a [Destinations] route; all screens are stubs until
 * the design system finalizes.
 */
@Composable
fun AppNavGraph(
    navController: NavHostController = rememberNavController(),
) {
    NavHost(
        navController = navController,
        startDestination = Destinations.TODAY.route,
    ) {
        composable(Destinations.TODAY.route) { TodayScreen() }
        composable(Destinations.PLAN.route) { PlanScreen() }
        composable(Destinations.PROJECTS.route) { ProjectsScreen() }
        composable(Destinations.TIME.route) { TimeScreen() }
        composable(Destinations.SETTINGS.route) { SettingsScreen() }
    }
}
