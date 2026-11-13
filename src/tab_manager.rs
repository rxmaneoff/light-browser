//! Модуль управления жизненным циклом и политиками вкладок (`tab_manager.rs`).
//!
//! Отвечает за:
//! - Режимы работы с памятью: Manual (по умолчанию), Balanced, Aggressive;
//! - Определение пригодности вкладки к усыплению (eligibility);
//! - Защиту от усыпления важных вкладок (активная, загрузка, воспроизведение медиа, внутренние);
//! - Переходы состояний: Active, Idle, Sleeping, Waking, WakeFailed;
//!
//! ВАЖНО: данный модуль не дублирует коллекцию вкладок `TabManager` из `tabs.rs`,
//! а реализует только логику политик и жизненного цикла.

use std::time::{Duration, Instant};

use crate::tabs::TabId;

/// Режимы управления памятью и усыплением вкладок.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SleepMode {
    /// По умолчанию: автоматическое усыпление строго выключено.
    #[default]
    Manual,
    /// Усыпление неактивных вкладок после заданного тайм-аута бездействия.
    Balanced,
    /// Агрессивное освобождение памяти с ограничением числа активных вкладок.
    Aggressive,
}

impl SleepMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Balanced => "balanced",
            Self::Aggressive => "aggressive",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "manual" => Some(Self::Manual),
            "balanced" => Some(Self::Balanced),
            "aggressive" => Some(Self::Aggressive),
            _ => None,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Manual => "Ручной (Manual)",
            Self::Balanced => "Сбалансированный (Balanced)",
            Self::Aggressive => "Экономичный (Aggressive)",
        }
    }
}

/// Состояние жизненного цикла вкладки.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabLifeCycleState {
    #[default]
    Active,
    Idle,
    Sleeping,
    Waking,
    WakeFailed,
}

impl TabLifeCycleState {
    pub fn is_sleeping(self) -> bool {
        matches!(self, Self::Sleeping)
    }

    pub fn badge_symbol(self) -> Option<&'static str> {
        match self {
            Self::Sleeping => Some("💤"),
            Self::Waking => Some("⏳"),
            Self::WakeFailed => Some("⚠️"),
            _ => None,
        }
    }
}

/// Конфигурация модуля политик вкладок.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabManagerModuleConfig {
    pub enabled: bool,
    pub mode: SleepMode,
    /// Тайм-аут бездействия в минутах (для режима Balanced).
    pub idle_timeout_minutes: u32,
    /// Максимальное число активных веб-процессов (для режима Aggressive).
    pub max_active_tabs: usize,
}

impl Default for TabManagerModuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: SleepMode::Manual,
            idle_timeout_minutes: 15,
            max_active_tabs: 5,
        }
    }
}

/// Данные о вкладке, необходимые для принятия решения об усыплении.
#[derive(Debug, Clone)]
pub struct TabSleepCandidate {
    pub id: TabId,
    pub is_active: bool,
    pub is_internal: bool,
    pub is_loading: bool,
    pub is_playing_audio: bool,
    pub state: TabLifeCycleState,
    pub last_interaction: Instant,
}

impl TabSleepCandidate {
    /// Проверяет, допустимо ли в принципе усыплять данную вкладку.
    ///
    /// Правила безопасности:
    /// - Активная вкладка никогда не усыпляется;
    /// - Загружающаяся вкладка никогда не усыпляется;
    /// - Внутренние системные страницы (настройки, модули) не усыпляются;
    /// - Вкладка с воспроизведением аудио/медиа не усыпляется;
    /// - Уже спящая или пробуждающаяся вкладка пропускается.
    pub fn is_eligible_for_sleep(&self) -> bool {
        if self.is_active {
            return false;
        }
        if self.is_internal {
            return false;
        }
        if self.is_loading {
            return false;
        }
        if self.is_playing_audio {
            return false;
        }
        if self.state == TabLifeCycleState::Sleeping || self.state == TabLifeCycleState::Waking {
            return false;
        }

        true
    }
}

/// Политика принятия решений об усыплении вкладок.
pub struct TabPolicyEngine;

impl TabPolicyEngine {
    /// Определяет список вкладок, подлежащих усыплению.
    pub fn evaluate_candidates(
        candidates: &[TabSleepCandidate],
        config: &TabManagerModuleConfig,
        now: Instant,
    ) -> Vec<TabId> {
        // Если модуль выключен или режим Manual — автоматическое усыпление запрещено!
        if !config.enabled || config.mode == SleepMode::Manual {
            return Vec::new();
        }

        let timeout = Duration::from_secs(config.idle_timeout_minutes as u64 * 60);
        let mut to_sleep = Vec::new();

        match config.mode {
            SleepMode::Manual => {}
            SleepMode::Balanced => {
                for candidate in candidates {
                    if candidate.is_eligible_for_sleep()
                        && now.duration_since(candidate.last_interaction) >= timeout
                    {
                        to_sleep.push(candidate.id);
                    }
                }
            }
            SleepMode::Aggressive => {
                // В Aggressive усыпляются вкладки по тайм-ауту (сокращенному) ЛИБО если превышен лимит
                let aggressive_timeout =
                    Duration::from_secs((config.idle_timeout_minutes as u64 * 60) / 2);

                // Собираем всех кандидатов, пригодных для сна
                let mut eligible: Vec<&TabSleepCandidate> = candidates
                    .iter()
                    .filter(|c| c.is_eligible_for_sleep())
                    .collect();

                // Сортируем: сначала те, с кем дольше всего не взаимодействовали
                eligible.sort_by_key(|c| c.last_interaction);

                let active_count = candidates
                    .iter()
                    .filter(|c| c.state != TabLifeCycleState::Sleeping)
                    .count();

                for candidate in eligible {
                    let idle_time = now.duration_since(candidate.last_interaction);
                    let exceeds_limit =
                        active_count.saturating_sub(to_sleep.len()) > config.max_active_tabs;

                    if idle_time >= aggressive_timeout || exceeds_limit {
                        to_sleep.push(candidate.id);
                    }
                }
            }
        }

        to_sleep
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candidate(
        id: u64,
        is_active: bool,
        is_loading: bool,
        is_audio: bool,
        is_internal: bool,
        idle_mins: u64,
    ) -> TabSleepCandidate {
        let now = Instant::now();
        TabSleepCandidate {
            id: TabId(id),
            is_active,
            is_internal,
            is_loading,
            is_playing_audio: is_audio,
            state: TabLifeCycleState::Idle,
            last_interaction: now - Duration::from_secs(idle_mins * 60),
        }
    }

    #[test]
    fn test_manual_mode_never_auto_sleeps() {
        let config = TabManagerModuleConfig {
            enabled: true,
            mode: SleepMode::Manual,
            idle_timeout_minutes: 5,
            max_active_tabs: 2,
        };

        let now = Instant::now();
        let candidates = vec![
            make_candidate(1, false, false, false, false, 60),
            make_candidate(2, false, false, false, false, 120),
        ];

        let result = TabPolicyEngine::evaluate_candidates(&candidates, &config, now);
        assert!(
            result.is_empty(),
            "В режиме Manual автосон не должен срабатывать!"
        );
    }

    #[test]
    fn test_balanced_mode_respects_timeout_and_safety() {
        let config = TabManagerModuleConfig {
            enabled: true,
            mode: SleepMode::Balanced,
            idle_timeout_minutes: 10,
            max_active_tabs: 5,
        };

        let now = Instant::now();
        let candidates = vec![
            // Активная вкладка (даже если 30 минут без клика) — НЕЛЬЗЯ усыплять
            make_candidate(1, true, false, false, false, 30),
            // Загружающаяся вкладка — НЕЛЬЗЯ усыплять
            make_candidate(2, false, true, false, false, 30),
            // Вкладка со звуком/медиа — НЕЛЬЗЯ усыплять
            make_candidate(3, false, false, true, false, 30),
            // Внутренняя вкладка — НЕЛЬЗЯ усыплять
            make_candidate(4, false, false, false, true, 30),
            // Обычная вкладка, 5 минут бездействия (< 10) — еще рано
            make_candidate(5, false, false, false, false, 5),
            // Обычная вкладка, 15 минут бездействия (>= 10) — УСЫПЛЯЕМ!
            make_candidate(6, false, false, false, false, 15),
        ];

        let result = TabPolicyEngine::evaluate_candidates(&candidates, &config, now);
        assert_eq!(result, vec![TabId(6)]);
    }

    #[test]
    fn test_disabled_module_does_nothing() {
        let config = TabManagerModuleConfig {
            enabled: false,
            mode: SleepMode::Balanced,
            idle_timeout_minutes: 1,
            max_active_tabs: 1,
        };

        let now = Instant::now();
        let candidates = vec![make_candidate(1, false, false, false, false, 60)];
        assert!(TabPolicyEngine::evaluate_candidates(&candidates, &config, now).is_empty());
    }
}
