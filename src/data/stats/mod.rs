use bevy::{prelude::*, sprite::Anchor};
use std::time::Duration;

use crate::{
    entities::text::{Cell, Column, Row, Table, TextContent},
    scenes::dilemma::{dilemma::DilemmaOptionConsequences, lever::LeverState},
    systems::colors::PRIMARY_COLOR,
};

pub struct StatsPlugin;
impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameStats::default())
            .insert_resource(DilemmaStats::default())
            .insert_resource(DilemmaRunStatsScope::default());
    }
}

#[derive(Clone)]
pub struct Decision {
    pub time: Duration,
    pub choice: LeverState,
}

#[derive(Resource, Clone)]
pub struct DilemmaStats {
    pub decisions: Vec<Decision>,
    pub result: Option<LeverState>,
    pub decision_time_available: Duration,
    pub decision_time_used: Duration,
    pub num_fatalities: usize,
    pub num_decisions: usize,
    pub average_num_decisions_per_second: Option<f64>,
    pub duration_before_first_decision: Option<Duration>,
    pub duration_remaining_at_last_decision: Option<Duration>,
}

impl Default for DilemmaStats {
    fn default() -> Self {
        Self {
            result: None,
            decisions: vec![],
            decision_time_available: Duration::ZERO,
            decision_time_used: Duration::ZERO,
            num_fatalities: 0,
            num_decisions: 0,
            average_num_decisions_per_second: None,
            duration_before_first_decision: None,
            duration_remaining_at_last_decision: None,
        }
    }
}

impl DilemmaStats {
    pub fn update(&mut self, lever: &LeverState, timer: &Timer) {
        self.decisions.push(Decision {
            time: timer.elapsed(),
            choice: *lever,
        });
        self.num_decisions = self.decisions.len();
        self.decision_time_available = timer.duration();

        if self.duration_before_first_decision.is_none() {
            self.duration_before_first_decision = Some(timer.elapsed());
        }

        self.duration_remaining_at_last_decision = Some(timer.remaining());
    }

    pub fn new(decision_time_available: Duration) -> Self {
        Self {
            result: None,
            decisions: vec![],
            decision_time_available,
            decision_time_used: Duration::ZERO,
            num_fatalities: 0,
            num_decisions: 0,
            average_num_decisions_per_second: None,
            duration_before_first_decision: None,
            duration_remaining_at_last_decision: None,
        }
    }

    pub fn finalize(
        &mut self,
        consequence: &DilemmaOptionConsequences,
        lever: &LeverState,
        timer: &Timer,
    ) {
        self.result = Some(*lever);
        self.num_fatalities = consequence.total_fatalities;
        self.decision_time_available = timer.duration();
        self.decision_time_used = timer.elapsed();

        if self.num_decisions > 0 {
            self.average_num_decisions_per_second =
                Some(self.num_decisions as f64 / timer.elapsed().as_secs_f64());
        }
    }

    pub fn to_table(&self) -> Table {
        // Compute the formatted strings just as before.
        let final_decision_str = match self.result {
            Some(ref decision) => format!("{:?}", decision),
            None => "None".to_string(),
        };

        let avg_decisions_str = self
            .average_num_decisions_per_second
            .map(|avg| format!("{:.2} Hz", avg))
            .unwrap_or_else(|| "N/A".to_string());
        let before_first_str = self
            .duration_before_first_decision
            .map(|d| format!("{:.2} s", d.as_secs_f64()))
            .unwrap_or_else(|| "N/A".to_string());
        let remaining_last_str = self
            .duration_remaining_at_last_decision
            .map(|d| format!("{:.2} s", d.as_secs_f64()))
            .unwrap_or_else(|| "N/A".to_string());
        let total_time_used_str = format!(
            "{:.2} s / {:.2} s",
            self.decision_time_used.as_secs_f64(),
            self.decision_time_available.as_secs_f64(),
        );

        // Create the cells for the left (label) column.
        let left_cells = vec![
            Cell::new(TextContent::new(
                String::from("Final Decision:"),
                PRIMARY_COLOR,
                12.0,
            )),
            Cell::new(TextContent::new(
                String::from("Number of Fatalities:"),
                PRIMARY_COLOR,
                12.0,
            )),
            Cell::new(TextContent::new(
                String::from("Number of Lever Pulls:"),
                PRIMARY_COLOR,
                12.0,
            )),
            Cell::new(TextContent::new(
                String::from("Average Pull Rate:"),
                PRIMARY_COLOR,
                12.0,
            )),
            Cell::new(TextContent::new(
                String::from("Time Before First Pull:"),
                PRIMARY_COLOR,
                12.0,
            )),
            Cell::new(TextContent::new(
                String::from("Time Remaining at Last Pull:"),
                PRIMARY_COLOR,
                12.0,
            )),
            Cell::new(TextContent::new(
                String::from("Total Time Used:"),
                PRIMARY_COLOR,
                12.0,
            )),
        ];

        // Create the cells for the right (value) column.
        let right_cells = vec![
            Cell::new(TextContent::new(final_decision_str, PRIMARY_COLOR, 12.0)),
            Cell::new(TextContent::new(
                self.num_fatalities.to_string(),
                PRIMARY_COLOR,
                12.0,
            )),
            Cell::new(TextContent::new(
                self.num_decisions.to_string(),
                PRIMARY_COLOR,
                12.0,
            )),
            Cell::new(TextContent::new(avg_decisions_str, PRIMARY_COLOR, 12.0)),
            Cell::new(TextContent::new(before_first_str, PRIMARY_COLOR, 12.0)),
            Cell::new(TextContent::new(remaining_last_str, PRIMARY_COLOR, 12.0)),
            Cell::new(TextContent::new(total_time_used_str, PRIMARY_COLOR, 12.0)),
        ];

        // Set the column widths and cell padding.
        let left_column_width = 230.0;
        let right_column_width = 130.0;
        let padding = Vec2::new(5.0, 5.0);

        let rows = vec![Row { height: 20.0 }; left_cells.len()];

        let left_column = Column::new(
            left_cells,
            left_column_width,
            padding,
            Anchor::CENTER_RIGHT,
            false,
        );
        let right_column = Column::new(
            right_cells,
            right_column_width,
            padding,
            Anchor::CENTER_LEFT,
            false,
        );

        // Return the complete table.
        Table {
            columns: vec![left_column, right_column],
            rows,
        }
    }

    fn reset(&mut self) {
        self.decisions.clear();
        self.result = None;
        self.decision_time_available = Duration::ZERO;
        self.decision_time_used = Duration::ZERO;
        self.num_fatalities = 0;
        self.num_decisions = 0;
        self.average_num_decisions_per_second = None;
        self.duration_before_first_decision = None;
        self.duration_remaining_at_last_decision = None;
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct DilemmaRunStatsScope {
    pub start_index: usize,
    pub expected_stage_count: usize,
}

impl Default for DilemmaRunStatsScope {
    fn default() -> Self {
        Self {
            start_index: 0,
            expected_stage_count: 1,
        }
    }
}

impl DilemmaRunStatsScope {
    pub const fn new(start_index: usize, expected_stage_count: usize) -> Self {
        Self {
            start_index,
            expected_stage_count,
        }
    }

    pub fn range(self, total_entries: usize) -> std::ops::Range<usize> {
        let start = self.start_index.min(total_entries);
        let expected_count = self.expected_stage_count.max(1);
        let end = start.saturating_add(expected_count).min(total_entries);
        start..end
    }

    pub fn entries<'a>(self, stats: &'a GameStats) -> &'a [DilemmaStats] {
        let range = self.range(stats.dilemma_stats.len());
        &stats.dilemma_stats[range]
    }
}

#[derive(Resource)]
pub struct GameStats {
    pub num_dilemmas: usize,
    pub total_fatalities: usize,
    pub mean_fatalities: f64,
    pub total_decisions: usize,
    pub mean_decisions: f64,
    pub dilemma_stats: Vec<DilemmaStats>,
    pub overall_avg_pull_rate: Option<f64>,
    pub overall_avg_first_pull_time: Option<Duration>,
    pub overall_avg_time_remaining: Option<Duration>,
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            num_dilemmas: 0,
            total_fatalities: 0,
            mean_fatalities: 0.0,
            total_decisions: 0,
            mean_decisions: 0.0,
            dilemma_stats: vec![],
            overall_avg_pull_rate: None,
            overall_avg_first_pull_time: None,
            overall_avg_time_remaining: None,
        }
    }
}

impl GameStats {
    pub fn from_dilemma_stats(dilemma_stats: &[DilemmaStats]) -> Self {
        let mut summary = Self::default();
        for stats in dilemma_stats {
            summary.update(stats.clone());
        }
        summary
    }

    pub fn update(&mut self, new_dilemma_stats: DilemmaStats) {
        self.dilemma_stats.push(new_dilemma_stats);

        self.num_dilemmas = self.dilemma_stats.len();
        self.total_fatalities = self
            .dilemma_stats
            .iter()
            .fold(0, |acc, stats: &DilemmaStats| acc + stats.num_fatalities);
        self.mean_fatalities = self.total_fatalities as f64 / self.num_dilemmas as f64;
        self.total_decisions = self
            .dilemma_stats
            .iter()
            .fold(0, |acc, stats: &DilemmaStats| acc + stats.num_decisions);
        self.mean_decisions = self.total_decisions as f64 / self.num_dilemmas as f64;

        // Calculate overall average pull rate across dilemmas
        let pull_rates: Vec<f64> = self
            .dilemma_stats
            .iter()
            .filter_map(|ds| ds.average_num_decisions_per_second)
            .collect();
        self.overall_avg_pull_rate = if !pull_rates.is_empty() {
            Some(pull_rates.iter().sum::<f64>() / pull_rates.len() as f64)
        } else {
            None
        };

        // Calculate overall average time before the first pull
        let first_pull_times: Vec<f64> = self
            .dilemma_stats
            .iter()
            .filter_map(|ds| ds.duration_before_first_decision)
            .map(|d| d.as_secs_f64())
            .collect();
        self.overall_avg_first_pull_time = if !first_pull_times.is_empty() {
            Some(Duration::from_secs_f64(
                first_pull_times.iter().sum::<f64>() / first_pull_times.len() as f64,
            ))
        } else {
            None
        };

        // Calculate overall average time remaining at the last pull
        let remaining_times: Vec<f64> = self
            .dilemma_stats
            .iter()
            .filter_map(|ds| ds.duration_remaining_at_last_decision)
            .map(|d| d.as_secs_f64())
            .collect();
        self.overall_avg_time_remaining = if !remaining_times.is_empty() {
            Some(Duration::from_secs_f64(
                remaining_times.iter().sum::<f64>() / remaining_times.len() as f64,
            ))
        } else {
            None
        };
    }

    pub fn update_stats(mut stats: ResMut<GameStats>, mut dilemma_stats: ResMut<DilemmaStats>) {
        stats.update(dilemma_stats.clone());
        dilemma_stats.reset();
    }

    pub fn to_table(&self) -> Table {
        // Format the numeric values
        let mean_fatalities_str = format!("{:.2}", self.mean_fatalities);
        let mean_decisions_str = format!("{:.2}", self.mean_decisions);
        let avg_pull_rate_str = self
            .overall_avg_pull_rate
            .map(|r| format!("{:.2} Hz", r))
            .unwrap_or_else(|| "N/A".to_string());
        let avg_first_pull_time_str = self
            .overall_avg_first_pull_time
            .map(|d| format!("{:.2} s", d.as_secs_f64()))
            .unwrap_or_else(|| "N/A".to_string());
        let avg_time_remaining_str = self
            .overall_avg_time_remaining
            .map(|d| format!("{:.2} s", d.as_secs_f64()))
            .unwrap_or_else(|| "N/A".to_string());

        // Build the left (label) cells.
        let left_cells = vec![
            Cell::new(TextContent::new(
                String::from("Total Dilemmas:"),
                PRIMARY_COLOR,
                12.0,
            )),
            Cell::new(TextContent::new(
                String::from("Total Fatalities:"),
                PRIMARY_COLOR,
                12.0,
            )),
            Cell::new(TextContent::new(
                String::from("Average Fatalities per Dilemma:"),
                PRIMARY_COLOR,
                12.0,
            )),
            Cell::new(TextContent::new(
                String::from("Total Lever Pulls:"),
                PRIMARY_COLOR,
                12.0,
            )),
            Cell::new(TextContent::new(
                String::from("Average Pulls Per Dilemma:"),
                PRIMARY_COLOR,
                12.0,
            )),
            Cell::new(TextContent::new(
                String::from("Average Pull Rate:"),
                PRIMARY_COLOR,
                12.0,
            )),
            Cell::new(TextContent::new(
                String::from("Average Time Before First Pull:"),
                PRIMARY_COLOR,
                12.0,
            )),
            Cell::new(TextContent::new(
                String::from("Average Time Remaining at Last Pull:"),
                PRIMARY_COLOR,
                12.0,
            )),
        ];

        // Build the right (value) cells.
        let right_cells = vec![
            Cell::new(TextContent::new(
                self.num_dilemmas.to_string(),
                PRIMARY_COLOR,
                12.0,
            )),
            Cell::new(TextContent::new(
                self.total_fatalities.to_string(),
                PRIMARY_COLOR,
                12.0,
            )),
            Cell::new(TextContent::new(mean_fatalities_str, PRIMARY_COLOR, 12.0)),
            Cell::new(TextContent::new(
                self.total_decisions.to_string(),
                PRIMARY_COLOR,
                12.0,
            )),
            Cell::new(TextContent::new(mean_decisions_str, PRIMARY_COLOR, 12.0)),
            Cell::new(TextContent::new(avg_pull_rate_str, PRIMARY_COLOR, 12.0)),
            Cell::new(TextContent::new(
                avg_first_pull_time_str,
                PRIMARY_COLOR,
                12.0,
            )),
            Cell::new(TextContent::new(
                avg_time_remaining_str,
                PRIMARY_COLOR,
                12.0,
            )),
        ];

        // Define column widths and padding.
        let left_column_width = 280.0;
        let right_column_width = 60.0;
        let padding = Vec2::new(5.0, 5.0);

        let rows = vec![Row { height: 20.0 }; left_cells.len()];

        // Create the two columns.
        let left_column = Column::new(
            left_cells,
            left_column_width,
            padding,
            Anchor::CENTER_RIGHT,
            false,
        );
        let right_column = Column::new(
            right_cells,
            right_column_width,
            padding,
            Anchor::CENTER_LEFT,
            false,
        );

        // Build and return the table.
        Table {
            columns: vec![left_column, right_column],
            rows,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finalize_records_timer_duration_even_without_decisions() {
        let mut stats = DilemmaStats::default();
        let mut timer = Timer::new(Duration::from_secs_f32(10.0), TimerMode::Once);
        timer.tick(Duration::from_secs_f32(2.5));

        stats.finalize(
            &DilemmaOptionConsequences {
                total_fatalities: 3,
            },
            &LeverState::Left,
            &timer,
        );

        assert_eq!(stats.decision_time_available, timer.duration());
        assert_eq!(stats.decision_time_used, timer.elapsed());
    }

    #[test]
    fn reset_clears_timing_fields_between_stages() {
        let mut stats = DilemmaStats::new(Duration::from_secs(12));
        let mut timer = Timer::new(Duration::from_secs_f32(12.0), TimerMode::Once);
        timer.tick(Duration::from_secs_f32(4.0));

        stats.update(&LeverState::Right, &timer);
        stats.finalize(
            &DilemmaOptionConsequences {
                total_fatalities: 1,
            },
            &LeverState::Right,
            &timer,
        );
        stats.reset();

        assert_eq!(stats.decision_time_available, Duration::ZERO);
        assert_eq!(stats.decision_time_used, Duration::ZERO);
        assert_eq!(stats.num_decisions, 0);
        assert!(stats.duration_before_first_decision.is_none());
        assert!(stats.duration_remaining_at_last_decision.is_none());
    }

    #[test]
    fn run_scope_clamps_to_available_entries() {
        let scope = DilemmaRunStatsScope::new(5, 4);
        assert_eq!(scope.range(7), 5..7);
        assert_eq!(scope.range(3), 3..3);
    }
}
