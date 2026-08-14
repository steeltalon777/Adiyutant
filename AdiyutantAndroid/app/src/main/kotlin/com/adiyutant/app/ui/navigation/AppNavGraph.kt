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
import com.adiyutant.app.ui.planner.PlannerViewModel

/**
 * Navigation graph for the five-tab shell (ADR-0002, TZ раздел 5).
 *
 * Сигнатура Phase 2: [planner] — общий [PlannerViewModel] (Activity scope),
 * [onNavigate] — колбэк переключения вкладок для Agent CTA и кросс-экранных
 * переходов. Маршруты и порядок вкладок не меняются.
 */
@Composable
fun AppNavGraph(
    navController: NavHostController = rememberNavController(),
    planner: PlannerViewModel,
    onNavigate: (Destinations) -> Unit,
) {
    NavHost(
        navController = navController,
        startDestination = Destinations.TODAY.route,
    ) {
        composable(Destinations.TODAY.route) {
            TodayScreen(planner = planner, onNavigate = onNavigate)
        }
        composable(Destinations.PLAN.route) {
            PlanScreen(planner = planner, onNavigate = onNavigate)
        }
        composable(Destinations.PROJECTS.route) { ProjectsScreen() }
        composable(Destinations.TIME.route) { TimeScreen() }
        composable(Destinations.SETTINGS.route) { SettingsScreen() }
    }
}
