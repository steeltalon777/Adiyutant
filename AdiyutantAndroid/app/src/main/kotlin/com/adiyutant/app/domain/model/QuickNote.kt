package com.adiyutant.app.domain.model

/**
 * Быстрая заметка (раздел 3.7 ТЗ). Хранится не более 5 (последняя первой),
 * отображается 3. [contextLabel] — заголовок текущего блока или label
 * активной сессии.
 */
data class QuickNote(
    val noteId: String,
    val text: String,           // непустой
    val kind: NoteKind,
    val atMs: Long,
    val contextLabel: String?,
)
