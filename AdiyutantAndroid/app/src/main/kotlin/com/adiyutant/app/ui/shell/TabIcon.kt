package com.adiyutant.app.ui.shell

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.addPathNodes
import androidx.compose.ui.unit.dp
import com.adiyutant.app.ui.navigation.Destinations

/**
 * Tab icons reconstructed from the SVG paths in the approved design
 * (docs/design/today-screen.html tabbar). Stroked, 24×24 viewport — the
 * NavigationBarItem tints them via Icon color, matching design semantics.
 */
object TabIcon {

    private fun icon(name: String, pathData: String, strokeWidth: Float = 1.7f): ImageVector =
        ImageVector.Builder(
            name = name,
            defaultWidth = 24.dp,
            defaultHeight = 24.dp,
            viewportWidth = 24f,
            viewportHeight = 24f,
        ).apply {
            addPath(
                pathData = addPathNodes(pathData),
                fill = null,
                stroke = SolidColor(Color.Black),
                strokeLineWidth = strokeWidth,
                strokeLineCap = androidx.compose.ui.graphics.StrokeCap.Round,
                strokeLineJoin = androidx.compose.ui.graphics.StrokeJoin.Round,
            )
        }.build()

    // Сегодня — "home" glyph
    private val Today = icon(
        name = "Today",
        pathData = "M3 12 12 3l9 9 M5 10v10h14V10",
    )

    // План — calendar with tick marks
    private val Plan = icon(
        name = "Plan",
        pathData = "M3 7.5a2.5 2.5 0 0 1 2.5-2.5h13a2.5 2.5 0 0 1 2.5 2.5v11a2.5 2.5 0 0 1-2.5 2.5h-13a2.5 2.5 0 0 1-2.5-2.5z M3 10h18 M8 3v4 M16 3v4",
    )

    // Проекты — folder
    private val Projects = icon(
        name = "Projects",
        pathData = "M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7z",
    )

    // Время — clock
    private val Time = icon(
        name = "Time",
        pathData = "M12 3a9 9 0 1 1 0 18 9 9 0 0 1 0-18z M12 7v5l3 2",
        strokeWidth = 1.7f,
    )

    // Настройки — gear (from design path)
    private val Settings = icon(
        name = "Settings",
        pathData = "M12 9a3 3 0 1 1 0 6 3 3 0 0 1 0-6z " +
            "M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z",
        strokeWidth = 1.7f,
    )

    fun forDestination(destination: Destinations): ImageVector = when (destination) {
        Destinations.TODAY -> Today
        Destinations.PLAN -> Plan
        Destinations.PROJECTS -> Projects
        Destinations.TIME -> Time
        Destinations.SETTINGS -> Settings
    }
}
